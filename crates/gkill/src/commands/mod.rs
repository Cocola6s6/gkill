pub mod install;
pub mod update;
pub mod search;
pub mod publish;
pub mod add;
pub mod remove;
pub mod login;
pub mod logout;
pub mod whoami;

use gkill_core::agents::{self, AgentConfig};
use gkill_core::config::DEFAULT_REGISTRY;
use anyhow::Result;
use dialoguer::Select;

pub fn resolve_agent(agent_opt: Option<String>) -> Result<AgentConfig> {
    if let Some(name) = agent_opt {
        return agents::get_agent(&name);
    }
    let all = agents::all_agents();
    let names: Vec<String> = all.iter().map(|a| format!("{} ({})", a.display_name, a.id)).collect();
    let idx = Select::new()
        .with_prompt("选择 Agent")
        .items(&names)
        .default(0)
        .interact()?;
    Ok(all[idx].clone())
}

pub fn resolve_mode(mode_opt: Option<String>) -> Result<String> {
    if let Some(m) = mode_opt {
        return Ok(m);
    }
    let opts = ["global - 全局安装", "project - 项目安装"];
    let idx = Select::new()
        .with_prompt("选择安装模式")
        .items(&opts)
        .default(0)
        .interact()?;
    Ok(if idx == 0 { "global".to_string() } else { "project".to_string() })
}

pub fn registry(opt: Option<String>) -> String {
    opt.unwrap_or_else(|| DEFAULT_REGISTRY.to_string())
}
