use gkill_core::agents::AgentConfig;
use gkill_core::config::DEFAULT_REGISTRY;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct AuthStatus {
    pub logged_in: bool,
    pub registry: String,
    pub user: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct InstalledInfo {
    pub slug: String,
    pub namespace: String,
    pub version: String,
    pub published_at: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct AgentInfo {
    pub id: String,
    pub display_name: String,
}

pub fn client(registry: Option<String>) -> gkill_core::http::Client {
    let reg = registry.unwrap_or_else(|| DEFAULT_REGISTRY.to_string());
    let token = gkill_core::config::read_token().ok().flatten();
    gkill_core::http::Client::new(&reg, token)
}

pub fn agent_from_id(id: &str) -> Result<AgentConfig, String> {
    gkill_core::agents::get_agent(id).map_err(|e| e.to_string())
}

pub mod auth;
pub mod publish;
pub mod skills;
