use super::client;
use gkill_core::publish;

#[tauri::command]
pub async fn publish_skill(
    path: String,
    namespace: String,
    visibility: String,
    registry: Option<String>,
) -> Result<(), String> {
    let c = client(registry);
    let dir = std::path::Path::new(&path);
    let slug = dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("skill")
        .to_string();
    let zip_data = publish::zip_skill_dir(dir).map_err(|e| e.to_string())?;
    gkill_core::api::publish_skill(&c, &namespace, &visibility, zip_data, &slug)
        .await
        .map_err(|e| e.to_string())
}

