use anyhow::Result;
use clap::Args;
use gkill_core::install;

#[derive(Args)]
pub struct RemoveArgs {
    /// Skill slug
    pub slug: String,
    /// 目标 Agent
    #[arg(long)]
    pub agent: Option<String>,
    /// 安装模式 global|project
    #[arg(long)]
    pub mode: Option<String>,
}

pub fn run(args: RemoveArgs) -> Result<()> {
    let agent = super::resolve_agent(args.agent)?;
    let mode = super::resolve_mode(args.mode)?;
    install::remove_skill(&agent, &mode, &args.slug)?;
    println!("✅ 已删除 {}", args.slug);
    Ok(())
}
