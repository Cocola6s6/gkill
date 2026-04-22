use serde::{Deserialize, Serialize};
use sycamore::prelude::*;

// ─── Mirror types (match gkill-core serialisation) ──────────────────────

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SkillItem {
    pub slug: String,
    pub display_name: Option<String>,
    pub namespace: Option<String>,
    pub summary: Option<String>,
    pub download_count: Option<u64>,
    pub author: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SkillPage {
    pub items: Vec<SkillItem>,
    pub total: u64,
    pub size: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct InstalledInfo {
    pub slug: String,
    pub namespace: String,
    pub version: String,
    pub published_at: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct AgentInfo {
    pub id: String,
    pub display_name: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct UpdateCandidate {
    pub slug: String,
    pub namespace: String,
    pub local_published_at: String,
    pub remote_published_at: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct AuthStatus {
    pub logged_in: bool,
    pub registry: String,
    pub user: Option<String>,
}

// ─── Global app state ──────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Page {
    Search,
    Installed,
    Publish,
    Settings,
}

#[derive(Clone)]
pub struct AppCtx {
    pub page: Signal<Page>,
    pub auth: Signal<AuthStatus>,
    pub agent: Signal<String>,
    pub mode: Signal<String>,
    pub toast: Signal<Option<String>>,
}

impl AppCtx {
    pub fn new() -> Self {
        Self {
            page: create_signal(Page::Search),
            auth: create_signal(AuthStatus {
                logged_in: false,
                registry: "https://skills.gydev.cn".to_string(),
                user: None,
            }),
            agent: create_signal("claude-code".to_string()),
            mode: create_signal("global".to_string()),
            toast: create_signal(None),
        }
    }
}
