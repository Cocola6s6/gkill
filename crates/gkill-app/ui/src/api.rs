use serde::Serialize;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use crate::state::{AgentInfo, AuthStatus, InstalledInfo, SkillPage, UpdateCandidate};

const DEFAULT_AGENT: &str = "claude-code";
const DEFAULT_MODE: &str = "global";

#[wasm_bindgen(inline_js = r#"
export function tauri_invoke(cmd, args) {
    return window.__TAURI_INTERNALS__.invoke(cmd, args);
}
"#)]
extern "C" {
    #[wasm_bindgen(catch)]
    fn tauri_invoke(cmd: &str, args: JsValue) -> Result<js_sys::Promise, JsValue>;
}

async fn invoke<A: Serialize, R: serde::de::DeserializeOwned>(cmd: &str, args: A) -> Result<R, String> {
    let args_js = serde_wasm_bindgen::to_value(&args).map_err(|e| e.to_string())?;
    let promise = tauri_invoke(cmd, args_js).map_err(|e| format!("{e:?}"))?;
    let result = JsFuture::from(promise).await.map_err(|e| {
        e.as_string().unwrap_or_else(|| format!("{e:?}"))
    })?;
    serde_wasm_bindgen::from_value(result).map_err(|e| e.to_string())
}

async fn invoke_void<A: Serialize>(cmd: &str, args: A) -> Result<(), String> {
    let args_js = serde_wasm_bindgen::to_value(&args).map_err(|e| e.to_string())?;
    let promise = tauri_invoke(cmd, args_js).map_err(|e| format!("{e:?}"))?;
    JsFuture::from(promise).await.map_err(|e| {
        e.as_string().unwrap_or_else(|| format!("{e:?}"))
    })?;
    Ok(())
}

#[derive(Serialize)]
struct Empty {}

pub async fn get_auth_status() -> Result<AuthStatus, String> {
    #[derive(Serialize)] struct A { registry: Option<String> }
    invoke("get_auth_status", A { registry: None }).await
}

pub async fn get_my_namespaces() -> Result<Vec<String>, String> {
    #[derive(Serialize)] struct A { registry: Option<String> }
    invoke("get_my_namespaces", A { registry: None }).await
}

pub async fn login(token: &str) -> Result<(), String> {
    #[derive(Serialize)] struct A<'a> { token: &'a str }
    invoke_void("login", A { token }).await
}

pub async fn logout() -> Result<(), String> {
    invoke_void("logout", Empty {}).await
}

pub async fn list_agents() -> Result<Vec<AgentInfo>, String> {
    invoke("list_agents", Empty {}).await
}

pub async fn search_skills(query: &str, sort: &str, page: u32, size: u32) -> Result<SkillPage, String> {
    #[derive(Serialize)] struct A<'a> { query: &'a str, sort: &'a str, page: u32, size: u32, registry: Option<String> }
    invoke("search_skills", A { query, sort, page, size, registry: None }).await
}

pub async fn install_skill(slug: &str, namespace: &str, agent: &str, mode: &str) -> Result<(), String> {
    #[derive(Serialize)] struct A<'a> { slug: &'a str, namespace: &'a str, agent: &'a str, mode: &'a str, version: Option<String>, registry: Option<String> }
    invoke_void("install_skill", A { slug, namespace, agent, mode, version: None, registry: None }).await
}

pub async fn list_installed() -> Result<Vec<InstalledInfo>, String> {
    #[derive(Serialize)] struct A<'a> { agent: &'a str, mode: &'a str }
    invoke("list_installed", A { agent: DEFAULT_AGENT, mode: DEFAULT_MODE }).await
}

pub async fn remove_skill(slug: &str, namespace: &str) -> Result<(), String> {
    #[derive(Serialize)] struct A<'a> { slug: &'a str, agent: &'a str, mode: &'a str }
    // namespace not used by remove command, but keep consistent
    let _ = namespace;
    invoke_void("remove_skill", A { slug, agent: DEFAULT_AGENT, mode: DEFAULT_MODE }).await
}

pub async fn find_updates() -> Result<Vec<UpdateCandidate>, String> {
    #[derive(Serialize)] struct A<'a> { agent: &'a str, mode: &'a str, registry: Option<String> }
    invoke("find_updates", A { agent: DEFAULT_AGENT, mode: DEFAULT_MODE, registry: None }).await
}

pub async fn update_skill(slug: &str, namespace: &str) -> Result<(), String> {
    #[derive(Serialize)] struct A<'a> { slug: &'a str, namespace: &'a str, agent: &'a str, mode: &'a str, registry: Option<String> }
    invoke_void("update_skill", A { slug, namespace, agent: DEFAULT_AGENT, mode: DEFAULT_MODE, registry: None }).await
}

pub async fn publish_skill(path: &str, namespace: &str, visibility: &str) -> Result<(), String> {
    #[derive(serde::Serialize)] struct A<'a> { path: &'a str, namespace: &'a str, visibility: &'a str, registry: Option<String> }
    invoke_void("publish_skill", A { path, namespace, visibility, registry: None }).await
}

pub async fn pick_folder() -> Result<Option<String>, String> {
    #[derive(serde::Serialize)] struct Empty {}
    let s: String = invoke("pick_folder", Empty {}).await?;
    if s.is_empty() { Ok(None) } else { Ok(Some(s)) }
}
