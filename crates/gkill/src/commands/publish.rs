use anyhow::Result;
use clap::Args;
use dialoguer::Select;
use gkill_core::{api, config, http::Client, publish};
use indicatif::{ProgressBar, ProgressStyle};
use std::path::PathBuf;

#[derive(Args)]
pub struct PublishArgs {
    /// skill 目录路径（不传则使用当前目录）
    pub path: Option<PathBuf>,
    /// 发布命名空间
    #[arg(long, default_value = "global")]
    pub ns: String,
    /// 可见性 PUBLIC|NAMESPACE_ONLY|PRIVATE
    #[arg(long)]
    pub visibility: Option<String>,
}

pub async fn run(args: PublishArgs) -> Result<()> {
    let token = config::read_token()?.ok_or_else(|| anyhow::anyhow!("请先运行 gkill login"))?;
    let client = Client::new(gkill_core::config::DEFAULT_REGISTRY, Some(token));

    let root = args.path.unwrap_or_else(|| std::env::current_dir().unwrap_or_default());
    let skills = publish::discover_skills(&root);

    if skills.is_empty() {
        anyhow::bail!("未找到包含 SKILL.md 的目录: {}", root.display());
    }

    let visibility = if let Some(v) = args.visibility {
        v
    } else {
        let opts = ["PUBLIC - 公开", "NAMESPACE_ONLY - 命名空间内", "PRIVATE - 私有"];
        let idx = Select::new()
            .with_prompt("选择可见性")
            .items(&opts)
            .default(0)
            .interact()?;
        ["PUBLIC", "NAMESPACE_ONLY", "PRIVATE"][idx].to_string()
    };

    // If multiple skills found, confirm batch publish
    if skills.len() > 1 {
        println!("发现 {} 个 skill，批量发布到 namespace: {}", skills.len(), args.ns);
        let confirm = dialoguer::Confirm::new()
            .with_prompt("确认批量发布？")
            .interact()?;
        if !confirm { return Ok(()); }
    }

    for skill_dir in &skills {
        let slug = skill_dir
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");
        let pb = spinner(&format!("正在打包并发布 {}...", slug));
        let archive = publish::zip_skill_dir(skill_dir)?;
        api::publish_skill(&client, &args.ns, &visibility, archive, slug).await?;
        pb.finish_and_clear();
        println!("✅ 已发布 {} → {}/{}", slug, args.ns, slug);
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
