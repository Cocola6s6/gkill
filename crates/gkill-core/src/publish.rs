use anyhow::{bail, Result};
use bytes::Bytes;
use std::io::Write;
use std::path::{Path, PathBuf};
use zip::write::SimpleFileOptions;

/// Recursively discover directories containing SKILL.md.
pub fn discover_skills(root: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    // Check root itself
    if root.join("SKILL.md").exists() {
        found.push(root.to_path_buf());
        return found; // root is a skill, don't recurse further
    }
    // Check immediate subdirs
    if let Ok(entries) = std::fs::read_dir(root) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() && path.join("SKILL.md").exists() {
                found.push(path);
            }
        }
    }
    found
}

/// Zip all files under skill_dir into a Bytes archive.
pub fn zip_skill_dir(skill_dir: &Path) -> Result<Bytes> {
    let buf = Vec::new();
    let cursor = std::io::Cursor::new(buf);
    let mut zip = zip::ZipWriter::new(cursor);
    let options = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    let dir_name = skill_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("skill");

    add_dir_to_zip(&mut zip, skill_dir, skill_dir, dir_name, options)?;
    let cursor = zip.finish()?;
    Ok(Bytes::from(cursor.into_inner()))
}

fn add_dir_to_zip(
    zip: &mut zip::ZipWriter<std::io::Cursor<Vec<u8>>>,
    base: &Path,
    current: &Path,
    prefix: &str,
    options: SimpleFileOptions,
) -> Result<()> {
    for entry in std::fs::read_dir(current)? {
        let entry = entry?;
        let path = entry.path();
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default();

        // Skip hidden files and _meta.json
        if name.starts_with('.') || name == "_meta.json" {
            continue;
        }

        let zip_path = format!("{}/{}", prefix, name);

        if path.is_dir() {
            zip.add_directory(&zip_path, options)?;
            add_dir_to_zip(zip, base, &path, &zip_path, options)?;
        } else if path.is_file() {
            if !is_safe_zip_path(&zip_path) {
                bail!("不安全的路径: {}", zip_path);
            }
            zip.start_file(&zip_path, options)?;
            let data = std::fs::read(&path)?;
            zip.write_all(&data)?;
        }
    }
    Ok(())
}

fn is_safe_zip_path(path: &str) -> bool {
    path.split('/').all(|seg| !seg.is_empty() && seg != "..")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_discover_skills_root() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("SKILL.md"), "# test").unwrap();
        let skills = discover_skills(dir.path());
        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0], dir.path());
    }

    #[test]
    fn test_discover_skills_subdirs() {
        let dir = TempDir::new().unwrap();
        let sub = dir.path().join("my-skill");
        fs::create_dir(&sub).unwrap();
        fs::write(sub.join("SKILL.md"), "# test").unwrap();
        let skills = discover_skills(dir.path());
        assert_eq!(skills.len(), 1);
    }
}
