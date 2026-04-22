use super::{client, AuthStatus};
use gkill_core::config;

#[tauri::command]
pub fn login(token: String) -> Result<(), String> {
    config::write_token(&token).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn logout() -> Result<(), String> {
    config::clear_token().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_auth_status(registry: Option<String>) -> AuthStatus {
    let token = config::read_token().ok().flatten();
    let logged_in = token.is_some();
    let reg = registry.unwrap_or_else(|| gkill_core::config::DEFAULT_REGISTRY.to_string());
    let user = if logged_in {
        let c = client(Some(reg.clone()));
        gkill_core::api::whoami(&c).await.ok().and_then(|v| {
            v.get("user")
                .and_then(|u| u.get("handle"))
                .and_then(|h| h.as_str().map(|s| s.to_string()))
        })
    } else {
        None
    };
    AuthStatus { logged_in, registry: reg, user }
}

#[tauri::command]
pub async fn whoami(registry: Option<String>) -> Result<serde_json::Value, String> {
    let c = client(registry);
    gkill_core::api::whoami(&c)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_my_namespaces(registry: Option<String>) -> Result<Vec<String>, String> {
    let c = client(registry);
    let val: serde_json::Value = gkill_core::api::my_namespaces(&c)
        .await
        .map_err(|e| e.to_string())?;
    // Response is ApiResponse<List<MyNamespaceResponse>>, extract data[].slug
    let slugs = val
        .get("data")
        .and_then(|d| d.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|item| item.get("slug").and_then(|s| s.as_str()).map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();
    Ok(slugs)
}
