mod commands;
use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "gkill", about = "SkillHub CLI - 管理 Agent Skills", version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 安装 skill 到 Agent 目录
    Install(commands::install::InstallArgs),
    /// 更新已安装的 skill
    Update(commands::update::UpdateArgs),
    /// 搜索 SkillHub 中的 skill
    Search(commands::search::SearchArgs),
    /// 发布 skill 到 SkillHub
    Publish(commands::publish::PublishArgs),
    /// 从 GitHub 仓库添加 skill
    Add(commands::add::AddArgs),
    /// 删除已安装的 skill
    Remove(commands::remove::RemoveArgs),
    /// 登录 SkillHub
    Login(commands::login::LoginArgs),
    /// 登出 SkillHub
    Logout,
    /// 查看当前登录用户
    Whoami(commands::whoami::WhoamiArgs),
}

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("\x1b[31m错误: {}\x1b[0m", e);
        std::process::exit(1);
    }
}

async fn run() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Install(args) => commands::install::run(args).await,
        Commands::Update(args) => commands::update::run(args).await,
        Commands::Search(args) => commands::search::run(args).await,
        Commands::Publish(args) => commands::publish::run(args).await,
        Commands::Add(args) => commands::add::run(args).await,
        Commands::Remove(args) => commands::remove::run(args),
        Commands::Login(args) => commands::login::run(args),
        Commands::Logout => commands::logout::run(),
        Commands::Whoami(args) => commands::whoami::run(args).await,
    }
}
