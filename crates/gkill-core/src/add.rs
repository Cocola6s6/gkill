use anyhow::{bail, Result};
use bytes::Bytes;
use std::io::Read;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct GitHubSkill {
    pub name: String,
    pub description: String,
    pub files: Vec<(String, Vec<u8>)>, // (relative_path, content)
}

/// Parse "owner/repo" or "https://github.com/owner/repo[.git]"
pub fn parse_github_source(src: &str) -> Result<(String, String)> {
    let src = src.trim().trim_end_matches('/').trim_end_matches(".git");
    if let Some(rest) = src.strip_prefix("https://github.com/") {
        let parts: Vec<&str> = rest.splitn(2, '/').collect();
        if parts.len() == 2 && !parts[0].is_empty() && !parts[1].is_empty() {
            return Ok((parts[0].to_string(), parts[1].to_string()));
        }
    } else {
        let parts: Vec<&str> = src.splitn(2, '/').collect();
        if parts.len() == 2 && !parts[0].is_empty() && !parts[1].is_empty() {
            return Ok((parts[0].to_string(), parts[1].to_string()));
        }
    }
    bail!("无效的 GitHub 来源: {}，格式为 owner/repo 或 https://github.com/owner/repo", src);
}

/// Download GitHub repo archive as ZIP bytes.
pub async fn fetch_github_zip(owner: &str, repo: &str) -> Result<Bytes> {
    let url = format!("https://github.com/{}/{}/archive/HEAD.zip", owner, repo);
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()?;
    let res = client.get(&url).send().await?;
    if !res.status().is_success() {
        bail!("下载 GitHub 仓库失败: HTTP {}", res.status());
    }
    Ok(res.bytes().await?)
}

/// Discover skills inside a GitHub repo ZIP.
/// A skill is a directory containing SKILL.md.
/// Searches: root level and skills/* subdirectories.
pub fn discover_github_skills(zip_data: &[u8]) -> Result<Vec<GitHubSkill>> {
    let cursor = std::io::Cursor::new(zip_data);
    let mut archive = zip::ZipArchive::new(cursor)?;

    // First pass: collect all files grouped by top-level-stripped paths
    let mut file_map: std::collections::HashMap<String, Vec<u8>> = std::collections::HashMap::new();
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let raw = file.name().to_string();
        // Strip repo root dir (e.g. "repo-HEAD/")
        let parts: Vec<&str> = raw.splitn(2, '/').collect();
        if parts.len() < 2 || parts[1].is_empty() {
            continue;
        }
        let rel = parts[1].to_string();
        if !file.is_dir() {
            let mut buf = Vec::new();
            file.read_to_end(&mut buf)?;
            file_map.insert(rel, buf);
        }
    }

    let mut skills = Vec::new();

    // Check if root is a skill
    if file_map.contains_key("SKILL.md") {
        let description = extract_description(file_map.get("SKILL.md").unwrap());
        let skill_files: Vec<(String, Vec<u8>)> = file_map
            .iter()
            .filter(|(k, _)| !k.contains('/') || k.starts_with("src/"))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        // Use all root-level files
        let all_files: Vec<(String, Vec<u8>)> = file_map
            .iter()
            .filter(|(k, _)| !k.starts_with("skills/"))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        skills.push(GitHubSkill {
            name: "root".to_string(),
            description,
            files: all_files,
        });
        let _ = skill_files;
    }

    // Check skills/* subdirectories
    let skill_dirs: std::collections::HashSet<String> = file_map
        .keys()
        .filter_map(|k| {
            let p: Vec<&str> = k.splitn(3, '/').collect();
            if p.len() >= 2 && p[0] == "skills" && !p[1].is_empty() {
                Some(p[1].to_string())
            } else {
                None
            }
        })
        .collect();

    for dir in skill_dirs {
        let skill_md_key = format!("skills/{}/SKILL.md", dir);
        if !file_map.contains_key(&skill_md_key) {
            continue;
        }
        let description = extract_description(file_map.get(&skill_md_key).unwrap());
        let prefix = format!("skills/{}/", dir);
        let files: Vec<(String, Vec<u8>)> = file_map
            .iter()
            .filter(|(k, _)| k.starts_with(&prefix))
            .map(|(k, v)| (k[prefix.len()..].to_string(), v.clone()))
            .collect();
        skills.push(GitHubSkill {
            name: dir,
            description,
            files,
        });
    }

    Ok(skills)
}

fn extract_description(skill_md: &[u8]) -> String {
    let content = String::from_utf8_lossy(skill_md);
    for line in content.lines() {
        let line = line.trim();
        if !line.is_empty() && !line.starts_with('#') {
            return line.chars().take(80).collect();
        }
    }
    String::new()
}

/// Copy a GitHub skill's files to the target agent directory.
pub fn install_github_skill(skill: &GitHubSkill, target_dir: &PathBuf) -> Result<()> {
    if target_dir.exists() {
        std::fs::remove_dir_all(target_dir)?;
    }
    std::fs::create_dir_all(target_dir)?;
    for (rel_path, content) in &skill.files {
        if !crate::install::is_safe_path(rel_path) {
            bail!("不安全的路径: {}", rel_path);
        }
        let out = target_dir.join(rel_path);
        if let Some(parent) = out.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(out, content)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_github_source_shorthand() {
        let (o, r) = parse_github_source("vercel-labs/skills").unwrap();
        assert_eq!(o, "vercel-labs");
        assert_eq!(r, "skills");
    }

    #[test]
    fn test_parse_github_source_url() {
        let (o, r) = parse_github_source("https://github.com/vercel-labs/skills").unwrap();
        assert_eq!(o, "vercel-labs");
        assert_eq!(r, "skills");
    }

    #[test]
    fn test_parse_github_source_git_url() {
        let (o, r) = parse_github_source("https://github.com/vercel-labs/skills.git").unwrap();
        assert_eq!(o, "vercel-labs");
        assert_eq!(r, "skills");
    }

    #[test]
    fn test_parse_github_source_invalid() {
        assert!(parse_github_source("not-valid").is_err());
    }
}
