use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

pub const DEFAULT_REGISTRY: &str = "https://skills.gydev.cn";

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Config {
    pub token: Option<String>,
}

pub fn config_path() -> PathBuf {
    let base = dirs::config_dir().unwrap_or_else(|| PathBuf::from("~/.config"));
    base.join("gkill").join("config.json")
}

pub fn read_token() -> Result<Option<String>> {
    let path = config_path();
    if !path.exists() {
        return Ok(None);
    }
    let content = fs::read_to_string(&path)
        .with_context(|| format!("读取配置文件失败: {}", path.display()))?;
    let cfg: Config = serde_json::from_str(&content).unwrap_or_default();
    Ok(cfg.token)
}

pub fn write_token(token: &str) -> Result<()> {
    let path = config_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let cfg = Config {
        token: Some(token.trim().to_string()),
    };
    let content = serde_json::to_string_pretty(&cfg)?;
    fs::write(&path, &content)?;
    // Set file permissions to 0o600 on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600))?;
    }
    Ok(())
}

pub fn clear_token() -> Result<()> {
    let path = config_path();
    if path.exists() {
        fs::remove_file(&path)?;
    }
    Ok(())
}
