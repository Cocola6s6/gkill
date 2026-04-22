use anyhow::Result;
use clap::Args;
use gkill_core::{config, http::Client, install};
use indicatif::{ProgressBar, ProgressStyle};

#[derive(Args)]
pub struct InstallArgs {
    /// Skill slug
    pub slug: String,
    /// Namespace（默认 global）
    #[arg(long, default_value = "global")]
    pub ns: String,
    /// 目标 Agent
    #[arg(long)]
    pub agent: Option<String>,
    /// 安装模式 global|project
    #[arg(long)]
    pub mode: Option<String>,
    /// 指定版本（不传则安装最新）
    #[arg(long)]
    pub version: Option<String>,
    /// SkillHub 地址
    #[arg(long)]
    pub registry: Option<String>,
}

pub async fn run(args: InstallArgs) -> Result<()> {
    let token = config::read_token()?;
    let registry = super::registry(args.registry);
    let client = Client::new(&registry, token);
    let agent = super::resolve_agent(args.agent)?;
    let mode = super::resolve_mode(args.mode)?;

    let pb = spinner(&format!("正在安装 {}...", args.slug));
    install::install_skill(
        &client,
        &args.slug,
        &args.ns,
        &agent,
        &mode,
        args.version.as_deref(),
    )
    .await?;
    pb.finish_and_clear();
    let dir = install::skill_dir(&agent, &mode, &args.slug);
    println!("✅ 已安装 {} → {}", args.slug, dir.display());
    Ok(())
}

fn spinner(msg: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(ProgressStyle::default_spinner().template("{spinner} {msg}").unwrap());
    pb.set_message(msg.to_string());
    pb.enable_steady_tick(std::time::Duration::from_millis(80));
    pb
}
