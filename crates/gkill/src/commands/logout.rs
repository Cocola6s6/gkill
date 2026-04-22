use anyhow::Result;
use gkill_core::config;

pub fn run() -> Result<()> {
    config::clear_token()?;
    println!("✅ 已登出，Token 已清除。");
    Ok(())
}
