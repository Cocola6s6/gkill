use anyhow::Result;
use clap::Args;
use gkill_core::config;

#[derive(Args)]
pub struct LoginArgs {
    /// 直接传入 API Token（不传则交互输入）
    #[arg(long)]
    pub token: Option<String>,
}

pub fn run(args: LoginArgs) -> Result<()> {
    let token = if let Some(t) = args.token {
        t
    } else {
        dialoguer::Password::new()
            .with_prompt("请输入 SkillHub API Token")
            .interact()?
    };
    config::write_token(&token)?;
    println!("✅ 登录成功，Token 已保存。");
    Ok(())
}
