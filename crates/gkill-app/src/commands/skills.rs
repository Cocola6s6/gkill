use super::{agent_from_id, client, AgentInfo, InstalledInfo};
use gkill_core::{agents, install, types::SkillPage, update};

#[tauri::command]
pub fn list_agents() -> Vec<AgentInfo> {
    agents::all_agents()
        .into_iter()
        .map(|a| AgentInfo {
            id: a.id.to_string(),
            display_name: a.display_name.to_string(),
        })
        .collect()
}

#[tauri::command]
pub async fn search_skills(
    query: String,
    sort: String,
    page: u32,
    size: u32,
    registry: Option<String>,
) -> Result<SkillPage, String> {
    let c = client(registry);
    gkill_core::api::search_skills(&c, &query, &sort, page, size)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn install_skill(
    slug: String,
    namespace: String,
    agent: String,
    mode: String,
    version: Option<String>,
    registry: Option<String>,
) -> Result<(), String> {
    let c = client(registry);
    let ag = agent_from_id(&agent)?;
    install::install_skill(&c, &slug, &namespace, &ag, &mode, version.as_deref())
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_installed(agent: String, mode: String) -> Result<Vec<InstalledInfo>, String> {
    let ag = agent_from_id(&agent)?;
    let base = if mode == "global" {
        ag.global_skills_dir.clone()
    } else {
        std::env::current_dir()
            .unwrap_or_default()
            .join(ag.skills_dir)
    };

    if !base.exists() {
        return Ok(vec![]);
    }

    let mut result = Vec::new();
    for entry in std::fs::read_dir(&base).map_err(|e| e.to_string())? {
        let path = entry.map_err(|e| e.to_string())?.path();
        if !path.is_dir() {
            continue;
        }
        if let Ok(Some(meta)) = install::read_meta(&path) {
            result.push(InstalledInfo {
                slug: meta.name,
                namespace: meta.namespace,
                version: meta.version,
                published_at: meta.published_at,
            });
        }
    }
    Ok(result)
}

#[tauri::command]
pub fn remove_skill(slug: String, agent: String, mode: String) -> Result<(), String> {
    let ag = agent_from_id(&agent)?;
    install::remove_skill(&ag, &mode, &slug).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn find_updates(
    agent: String,
    mode: String,
    registry: Option<String>,
) -> Result<Vec<update::UpdateCandidate>, String> {
    let c = client(registry);
    let ag = agent_from_id(&agent)?;
    update::find_updates(&c, &ag, &mode)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_skill(
    slug: String,
    namespace: String,
    agent: String,
    mode: String,
    registry: Option<String>,
) -> Result<(), String> {
    let c = client(registry);
    let ag = agent_from_id(&agent)?;
    install::install_skill(&c, &slug, &namespace, &ag, &mode, None)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_skill_markdown(
    slug: String,
    namespace: String,
    version: Option<String>,
    registry: Option<String>,
) -> Result<String, String> {
    let c = client(registry);
    install::fetch_skill_markdown(&c, &slug, &namespace, version.as_deref())
        .await
        .map_err(|e| e.to_string())
}
