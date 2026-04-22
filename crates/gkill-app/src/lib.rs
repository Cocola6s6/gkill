mod commands;

use commands::{auth, publish, skills};
use tauri_plugin_dialog::DialogExt;

#[tauri::command]
async fn pick_folder(app: tauri::AppHandle) -> String {
    let (tx, rx) = tokio::sync::oneshot::channel::<String>();
    app.dialog()
        .file()
        .pick_folder(move |path| {
            let s = path
                .and_then(|p| match p {
                    tauri_plugin_dialog::FilePath::Path(pb) => {
                        Some(pb.to_string_lossy().to_string())
                    }
                    tauri_plugin_dialog::FilePath::Url(u) => u
                        .to_file_path()
                        .ok()
                        .map(|pb| pb.to_string_lossy().to_string()),
                })
                .unwrap_or_default();
            let _ = tx.send(s);
        });
    rx.await.unwrap_or_default()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            pick_folder,
            // auth
            auth::login,
            auth::logout,
            auth::get_auth_status,
            auth::whoami,
            auth::get_my_namespaces,
            // skills
            skills::list_agents,
            skills::search_skills,
            skills::install_skill,
            skills::list_installed,
            skills::remove_skill,
            skills::find_updates,
            skills::update_skill,
            // publish
            publish::publish_skill,
        ])
        .run(tauri::generate_context!())
        .expect("error while running gkill app");
}
