mod commands;

use tokio::sync::Mutex;
use ns_download_engine::Engine;

pub struct AppState {
    pub engine: Mutex<Option<Engine>>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt().with_env_filter("info").init();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_process::init())
        .manage(AppState {
            engine: Mutex::new(None),
        })
        .invoke_handler(tauri::generate_handler![
            commands::init_engine,
            commands::get_tasks,
            commands::create_task,
            commands::pause_task,
            commands::resume_task,
            commands::remove_task,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
