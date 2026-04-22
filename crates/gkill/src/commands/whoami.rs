use anyhow::Result;
use clap::Args;
use gkill_core::{api, config, http::Client};
use indicatif::{ProgressBar, ProgressStyle};

#[derive(Args)]
pub struct WhoamiArgs {
    /// SkillHub 地址
    #[arg(long)]
    pub registry: Option<String>,
}

pub async fn run(args: WhoamiArgs) -> Result<()> {
    let token = config::read_token()?.ok_or_else(|| anyhow::anyhow!("请先运行 gkill login"))?;
    let registry = super::registry(args.registry);
    let client = Client::new(&registry, Some(token));
    let pb = spinner("正在获取用户信息...");
    let info = api::whoami(&client).await?;
    pb.finish_and_clear();
    println!("{}", serde_json::to_string_pretty(&info)?);
    Ok(())
}

fn spinner(msg: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(ProgressStyle::default_spinner().template("{spinner} {msg}").unwrap());
    pb.set_message(msg.to_string());
    pb.enable_steady_tick(std::time::Duration::from_millis(80));
    pb
}
