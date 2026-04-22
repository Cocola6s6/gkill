use anyhow::Result;
use clap::Args;
use dialoguer::MultiSelect;
use gkill_core::{config, http::Client, install, update};
use indicatif::{ProgressBar, ProgressStyle};

#[derive(Args)]
pub struct UpdateArgs {
    /// 只更新指定 slug（不传则检查全部）
    pub slug: Option<String>,
    /// 目标 Agent
    #[arg(long)]
    pub agent: Option<String>,
    /// 安装模式 global|project
    #[arg(long)]
    pub mode: Option<String>,
    /// SkillHub 地址
    #[arg(long)]
    pub registry: Option<String>,
}

pub async fn run(args: UpdateArgs) -> Result<()> {
    let token = config::read_token()?;
    let registry = super::registry(args.registry);
    let client = Client::new(&registry, token);
    let agent = super::resolve_agent(args.agent)?;
    let mode = super::resolve_mode(args.mode)?;

    let pb = spinner("正在检查更新...");
    let mut candidates = update::find_updates(&client, &agent, &mode).await?;
    pb.finish_and_clear();

    // Filter by slug if provided
    if let Some(slug) = &args.slug {
        candidates.retain(|c| &c.slug == slug);
    }

    if candidates.is_empty() {
        println!("暂无需要更新的 skill。");
        return Ok(());
    }

    let labels: Vec<String> = candidates
        .iter()
        .map(|c| format!("{} ({} → {})", c.slug, &c.local_published_at[..10], &c.remote_published_at[..10]))
        .collect();

    let selections = MultiSelect::new()
        .with_prompt("选择要更新的 skill（空格选中，回车确认）")
        .items(&labels)
        .interact()?;

    if selections.is_empty() {
        println!("未选择任何 skill。");
        return Ok(());
    }

    for idx in selections {
        let c = &candidates[idx];
        let pb = spinner(&format!("正在更新 {}...", c.slug));
        install::install_skill(&client, &c.slug, &c.namespace, &agent, &mode, None).await?;
        pb.finish_and_clear();
        println!("✅ 已更新 {}", c.slug);
    }
    Ok(())
}

fn spinner(msg: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(ProgressStyle::default_spinner().template("{spinner} {msg}").unwrap());
    pb.set_message(msg.to_string());
    pb.enable_steady_tick(std::time::Duration::from_millis(80));
    pb
}
