use crate::agents::AgentConfig;
use crate::api;
use crate::http::Client;
use crate::install::read_meta;
use anyhow::Result;
use std::fs;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UpdateCandidate {
    pub slug: String,
    pub namespace: String,
    pub local_published_at: String,
    pub remote_published_at: String,
}

/// Scan agent dir for installed skills and check for updates.
pub async fn find_updates(
    client: &Client,
    agent: &AgentConfig,
    mode: &str,
) -> Result<Vec<UpdateCandidate>> {
    let base = if mode == "global" {
        agent.global_skills_dir.clone()
    } else {
        std::env::current_dir()
            .unwrap_or_default()
            .join(agent.skills_dir)
    };

    if !base.exists() {
        return Ok(vec![]);
    }

    let mut candidates = Vec::new();
    for entry in fs::read_dir(&base)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let meta = match read_meta(&path) {
            Ok(Some(m)) => m,
            _ => continue, // skip if no _meta.json
        };

        // Fetch remote version
        let detail = match api::get_skill(client, &meta.namespace, &meta.name).await {
            Ok(d) => d,
            Err(_) => continue,
        };
        let remote_ver = match detail.published_version.or(detail.headline_version) {
            Some(v) => v.version,
            None => continue,
        };
        let ver_detail = match api::get_version(client, &meta.namespace, &meta.name, &remote_ver).await {
            Ok(d) => d,
            Err(_) => continue,
        };

        if is_newer(&ver_detail.published_at, &meta.published_at) {
            candidates.push(UpdateCandidate {
                slug: meta.name.clone(),
                namespace: meta.namespace.clone(),
                local_published_at: meta.published_at,
                remote_published_at: ver_detail.published_at,
            });
        }
    }

    Ok(candidates)
}

/// ISO 8601 lexicographic comparison: remote > local means newer.
pub fn is_newer(remote: &str, local: &str) -> bool {
    if !is_iso8601(remote) || !is_iso8601(local) {
        return false;
    }
    remote.trim() > local.trim()
}

fn is_iso8601(s: &str) -> bool {
    let s = s.trim();
    // Accept: 2026-04-15T12:00:00.000Z or 2026-04-15T12:00:00Z
    s.len() >= 20 && s.contains('T') && (s.ends_with('Z') || s.contains('+'))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_newer() {
        assert!(is_newer("2026-04-16T00:00:00.000Z", "2026-04-15T00:00:00.000Z"));
        assert!(!is_newer("2026-04-15T00:00:00.000Z", "2026-04-16T00:00:00.000Z"));
        assert!(!is_newer("2026-04-15T00:00:00.000Z", "2026-04-15T00:00:00.000Z"));
        assert!(!is_newer("not-a-date", "2026-04-15T00:00:00.000Z"));
    }
}
