mod api_server;
mod commands;
mod settings;
mod sink;

use std::sync::Arc;
use tokio::sync::Mutex;
use ns_download_engine::Engine;
#[cfg(not(mobile))]
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
#[cfg(not(mobile))]
use tauri::menu::{Menu, MenuItem};
#[cfg(not(mobile))]
use tauri::image::Image;
#[cfg(not(mobile))]
use tauri::Manager;

pub struct AppState {
    pub engine: Arc<Mutex<Option<Engine>>>,
    pub api_server_shutdown: Mutex<Option<tokio::sync::mpsc::Sender<crate::api_server::ServerCommand>>>,
}

#[cfg(not(mobile))]
fn setup_tray(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let icon_bytes = include_bytes!("../icons/tray-icon.png");
    let icon = Image::from_bytes(icon_bytes)?;

    let show = MenuItem::with_id(app, "show", "显示窗口", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show, &quit])?;

    TrayIconBuilder::new()
        .icon(icon)
        .icon_as_template(true)
        .tooltip("ns-download")
        .menu(&menu)
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event {
                if let Some(window) = tray.app_handle().get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
        })
        .on_menu_event(|app, event| {
            match event.id().as_ref() {
                "show" => {
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
                "quit" => {
                    app.exit(0);
                }
                _ => {}
            }
        })
        .build(app)?;

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt().with_env_filter("info").init();

    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_process::init());

    #[cfg(not(mobile))]
    let builder = builder.plugin(tauri_plugin_autostart::init(
        tauri_plugin_autostart::MacosLauncher::LaunchAgent,
        None,
    ));

    builder
        .manage(AppState {
            engine: Arc::new(Mutex::new(None)),
            api_server_shutdown: Mutex::new(None),
        })
        .invoke_handler(tauri::generate_handler![
            commands::init_engine,
            commands::get_tasks,
            commands::create_task,
            commands::pause_task,
            commands::resume_task,
            commands::remove_task,
            commands::load_settings,
            commands::save_settings,
            commands::test_proxy,
            commands::check_update,
            commands::check_command_exists,
            commands::export_logs,
            commands::start_api_server,
            commands::stop_api_server,
            commands::set_task_priority,
            commands::move_task_to_queue,
            commands::save_queues,
            commands::load_queues,
            commands::set_task_segments,
            commands::probe_torrent_file,
            commands::reveal_in_folder,
            commands::send_notification,
            commands::prevent_sleep,
            commands::shutdown_system,
            commands::submit_feedback,
        ])
        .setup(|app| {
            #[cfg(not(mobile))]
            if let Err(e) = setup_tray(app) {
                eprintln!("Failed to setup tray: {e}");
            }
            #[cfg(mobile)]
            let _ = app;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
