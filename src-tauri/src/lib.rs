mod commands;
mod settings;
mod sink;

use tokio::sync::Mutex;
use ns_download_engine::Engine;
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::menu::{Menu, MenuItem};
use tauri::image::Image;
use tauri::Manager;

pub struct AppState {
    pub engine: Mutex<Option<Engine>>,
}

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

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
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
            commands::reveal_in_folder,
            commands::send_notification,
            commands::prevent_sleep,
        ])
        .setup(|app| {
            if let Err(e) = setup_tray(app) {
                eprintln!("Failed to setup tray: {e}");
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
