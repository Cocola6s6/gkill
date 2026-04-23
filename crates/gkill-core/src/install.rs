use crate::agents::AgentConfig;
use crate::api;
use crate::http::Client;
use crate::types::SkillMeta;
use anyhow::{bail, Result};
use bytes::Bytes;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

pub fn skill_dir(agent: &AgentConfig, mode: &str, slug: &str) -> PathBuf {
    let base = if mode == "global" {
        agent.global_skills_dir.clone()
    } else {
        std::env::current_dir()
            .unwrap_or_default()
            .join(agent.skills_dir)
    };
    base.join(slug)
}

pub async fn install_skill(
    client: &Client,
    slug: &str,
    namespace: &str,
    agent: &AgentConfig,
    mode: &str,
    version: Option<&str>,
) -> Result<()> {
    // 1. Resolve version
    let detail = api::get_skill(client, namespace, slug).await?;
    let resolved_version = if let Some(v) = version {
        v.to_string()
    } else {
        detail
            .published_version
            .or(detail.headline_version)
            .map(|v| v.version)
            .ok_or_else(|| anyhow::anyhow!("skill '{}' 没有可用版本", slug))?
    };

    // 2. Get version detail
    let ver_detail = api::get_version(client, namespace, slug, &resolved_version).await?;

    // 3. Download ZIP
    let zip_data = api::download_skill(client, namespace, slug, &resolved_version).await?;

    // 4. Extract
    let target_dir = skill_dir(agent, mode, slug);
    if target_dir.exists() {
        fs::remove_dir_all(&target_dir)?;
    }
    fs::create_dir_all(&target_dir)?;
    extract_zip(&zip_data, &target_dir)?;

    // 5. Write _meta.json
    let meta = SkillMeta {
        version: resolved_version,
        name: slug.to_string(),
        namespace: namespace.to_string(),
        published_at: ver_detail.published_at,
    };
    write_meta(&target_dir, &meta)?;

    Ok(())
}

pub async fn fetch_skill_markdown(
    client: &Client,
    slug: &str,
    namespace: &str,
    version: Option<&str>,
) -> Result<String> {
    let detail = api::get_skill(client, namespace, slug).await?;
    let resolved_version = if let Some(v) = version {
        v.to_string()
    } else {
        detail
            .published_version
            .or(detail.headline_version)
            .map(|v| v.version)
            .ok_or_else(|| anyhow::anyhow!("skill '{}' 没有可用版本", slug))?
    };

    let zip_data = api::download_skill(client, namespace, slug, &resolved_version).await?;
    read_skill_markdown_from_zip(&zip_data)
}

pub fn remove_skill(agent: &AgentConfig, mode: &str, slug: &str) -> Result<()> {
    let dir = skill_dir(agent, mode, slug);
    if !dir.exists() {
        bail!("skill '{}' 不存在于 {}", slug, dir.display());
    }
    fs::remove_dir_all(&dir)?;
    Ok(())
}

pub fn write_meta(dir: &Path, meta: &SkillMeta) -> Result<()> {
    let path = dir.join("_meta.json");
    fs::write(path, serde_json::to_string_pretty(meta)?)?;
    Ok(())
}

pub fn read_meta(dir: &Path) -> Result<Option<SkillMeta>> {
    let path = dir.join("_meta.json");
    if !path.exists() {
        return Ok(None);
    }
    let content = fs::read_to_string(&path)?;
    Ok(Some(serde_json::from_str(&content)?))
}

fn detect_strip_prefix(archive: &mut zip::ZipArchive<std::io::Cursor<&[u8]>>) -> Option<String> {
    // Detect if every entry shares the same single top-level directory prefix.
    // If so, return that prefix so we can strip it during extraction.
    let mut prefix: Option<String> = None;
    for i in 0..archive.len() {
        let file = match archive.by_index(i) {
            Ok(f) => f,
            Err(_) => return None,
        };
        let name = file.name();
        let top = name.splitn(2, '/').next().unwrap_or("");
        if top.is_empty() {
            return None;
        }
        match &prefix {
            None => prefix = Some(top.to_string()),
            Some(p) if p != top => return None,
            _ => {}
        }
    }
    prefix
}

fn extract_zip(data: &Bytes, target_dir: &Path) -> Result<()> {
    let cursor = std::io::Cursor::new(data.as_ref());
    let mut archive = zip::ZipArchive::new(cursor)?;

    // Detect whether the ZIP wraps everything under a single top-level dir.
    let strip = {
        let cursor2 = std::io::Cursor::new(data.as_ref());
        let mut a2 = zip::ZipArchive::new(cursor2)?;
        detect_strip_prefix(&mut a2)
    };

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let raw_name = file.name().to_string();

        let rel: &str = if let Some(pfx) = &strip {
            // Strip the shared top-level prefix (e.g. "skill-name/")
            let prefix_slash = format!("{pfx}/");
            let stripped = raw_name.strip_prefix(prefix_slash.as_str()).unwrap_or("");
            if stripped.is_empty() {
                continue; // skip the top-level dir entry itself
            }
            stripped
        } else {
            // No wrapper — use paths as-is; skip bare directory entries
            if raw_name.ends_with('/') {
                continue;
            }
            raw_name.as_str()
        };

        if !is_safe_path(rel) {
            bail!("ZIP 包含不安全路径: {}", rel);
        }

        let out_path = target_dir.join(rel);
        if raw_name.ends_with('/') {
            fs::create_dir_all(&out_path)?;
        } else {
            if let Some(parent) = out_path.parent() {
                fs::create_dir_all(parent)?;
            }
            let mut buf = Vec::new();
            file.read_to_end(&mut buf)?;
            fs::write(&out_path, buf)?;
        }
    }
    Ok(())
}

fn read_skill_markdown_from_zip(data: &Bytes) -> Result<String> {
    let cursor = std::io::Cursor::new(data.as_ref());
    let mut archive = zip::ZipArchive::new(cursor)?;

    let strip = {
        let cursor2 = std::io::Cursor::new(data.as_ref());
        let mut a2 = zip::ZipArchive::new(cursor2)?;
        detect_strip_prefix(&mut a2)
    };

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        if file.name().ends_with('/') {
            continue;
        }

        let raw_name = file.name().to_string();
        let rel: &str = if let Some(pfx) = &strip {
            let prefix_slash = format!("{pfx}/");
            let stripped = raw_name.strip_prefix(prefix_slash.as_str()).unwrap_or("");
            if stripped.is_empty() {
                continue;
            }
            stripped
        } else {
            raw_name.as_str()
        };

        if !is_safe_path(rel) {
            bail!("ZIP 包含不安全路径: {}", rel);
        }

        if rel.eq_ignore_ascii_case("SKILL.md") {
            let mut buf = Vec::new();
            file.read_to_end(&mut buf)?;
            return Ok(String::from_utf8_lossy(&buf).into_owned());
        }
    }

    bail!("未在 skill 包中找到 SKILL.md")
}

pub fn is_safe_path(path: &str) -> bool {
    path.split('/')
        .all(|seg| !seg.is_empty() && seg != "..")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_path() {
        assert!(is_safe_path("foo/bar.md"));
        assert!(is_safe_path("SKILL.md"));
        assert!(!is_safe_path("../etc/passwd"));
        assert!(!is_safe_path("foo//bar"));
        assert!(!is_safe_path("foo/../bar"));
    }
}
