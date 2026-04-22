use anyhow::Result;
use clap::Args;
use dialoguer::MultiSelect;
use gkill_core::add;
use indicatif::{ProgressBar, ProgressStyle};

#[derive(Args)]
pub struct AddArgs {
    /// GitHub 来源（owner/repo 或 https://github.com/owner/repo）
    pub source: String,
    /// 目标 Agent
    #[arg(long)]
    pub agent: Option<String>,
    /// 安装模式 global|project
    #[arg(long)]
    pub mode: Option<String>,
}

pub async fn run(args: AddArgs) -> Result<()> {
    let (owner, repo) = add::parse_github_source(&args.source)?;
    let agent = super::resolve_agent(args.agent)?;
    let mode = super::resolve_mode(args.mode)?;

    let pb = spinner(&format!("正在下载 {}/{}...", owner, repo));
    let zip_data = add::fetch_github_zip(&owner, &repo).await?;
    pb.finish_and_clear();

    let pb = spinner("正在扫描 skill...");
    let skills = add::discover_github_skills(&zip_data)?;
    pb.finish_and_clear();

    if skills.is_empty() {
        anyhow::bail!("仓库中未发现任何包含 SKILL.md 的 skill");
    }

    let selected = if skills.len() == 1 {
        vec![0]
    } else {
        let labels: Vec<String> = skills
            .iter()
            .map(|s| {
                if s.description.is_empty() {
                    s.name.clone()
                } else {
                    format!("{} - {}", s.name, s.description)
                }
            })
            .collect();
        MultiSelect::new()
            .with_prompt("选择要安装的 skill")
            .items(&labels)
            .interact()?
    };

    if selected.is_empty() {
        println!("未选择任何 skill。");
        return Ok(());
    }

    for idx in selected {
        let skill = &skills[idx];
        let base = if mode == "global" {
            agent.global_skills_dir.clone()
        } else {
            std::env::current_dir()
                .unwrap_or_default()
                .join(agent.skills_dir)
        };
        let target = base.join(&skill.name);
        add::install_github_skill(skill, &target)?;
        println!("✅ 已安装 {} → {}", skill.name, target.display());
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
