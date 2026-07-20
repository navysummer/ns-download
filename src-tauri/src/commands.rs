use std::sync::Arc;
use std::collections::HashMap;
use tauri::State;
use ns_download_engine::{Engine, EngineConfig, NoopSelection};
use ns_download_engine::bt_downloader::BtConfig;
use ns_download_engine::proxy_config::ProxyConfig;
use ns_download_engine::model::TaskInfo;
use serde::{Deserialize, Serialize};
use crate::AppState;
use crate::settings;

#[derive(Serialize)]
pub struct TaskResponse {
    pub id: String,
    pub url: String,
    pub file_name: String,
    pub save_dir: String,
    pub status: i32,
    pub downloaded_bytes: i64,
    pub total_bytes: i64,
    pub error_message: String,
    pub created_at: String,
    pub completed_at: String,
    pub segments: i32,
}

impl From<TaskInfo> for TaskResponse {
    fn from(t: TaskInfo) -> Self {
        Self {
            id: t.task_id,
            url: t.url,
            file_name: t.file_name,
            save_dir: t.save_dir,
            status: t.status,
            downloaded_bytes: t.downloaded_bytes,
            total_bytes: t.total_bytes,
            error_message: t.error_message,
            created_at: t.created_at,
            completed_at: t.completed_at,
            segments: t.segments,
        }
    }
}

#[derive(Deserialize)]
pub struct CreateTaskSpec {
    pub url: String,
    pub save_dir: String,
    pub file_name: Option<String>,
    pub segments: Option<i32>,
}

#[tauri::command]
pub async fn init_engine(app_handle: tauri::AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    let mut engine_guard = state.engine.lock().await;
    if engine_guard.is_some() {
        return Ok(());
    }

    let app_data = app_data_dir();
    let default_save = dirs_or_fallback();

    let config = EngineConfig {
        max_concurrent: 5,
        speed_limit_bps: 0,
        default_save_dir: default_save.clone(),
        app_data_dir: app_data.clone(),
        bt_config: BtConfig::default(),
        proxy_config: ProxyConfig::default(),
        user_agent: String::new(),
        data_dir_override: None,
        database_url: None,
    };

    let sink = Arc::new(crate::sink::TauriEventSink::new(app_handle));

    let mut engine = Engine::new(config, sink, Arc::new(NoopSelection))
        .await
        .map_err(|e| e.to_string())?;

    // Initialize default config values in DB if not already set.
    engine.db.init_default_config(&default_save).await.map_err(|e| e.to_string())?;

    // Apply all saved settings from DB to the running engine.
    settings::apply_all(&mut engine).await;

    *engine_guard = Some(engine);
    Ok(())
}

#[tauri::command]
pub async fn get_tasks(state: State<'_, AppState>) -> Result<Vec<TaskResponse>, String> {
    let engine_guard = state.engine.lock().await;
    let engine = engine_guard.as_ref().ok_or("engine not initialized")?;

    let tasks = engine.db.load_all_tasks().await.map_err(|e| e.to_string())?;
    Ok(tasks.into_iter().map(TaskResponse::from).collect())
}

#[tauri::command]
pub async fn create_task(state: State<'_, AppState>, spec: CreateTaskSpec) -> Result<(), String> {
    let mut engine_guard = state.engine.lock().await;
    let engine = engine_guard.as_mut().ok_or("engine not initialized")?;

    let new_spec = ns_download_engine::download_manager::NewTaskSpec {
        url: spec.url,
        save_dir: spec.save_dir,
        file_name: spec.file_name.unwrap_or_default(),
        segments: spec.segments.unwrap_or(0),
        ..Default::default()
    };

    engine.manager.create_task(new_spec).await;
    Ok(())
}

#[tauri::command]
pub async fn pause_task(state: State<'_, AppState>, task_id: String) -> Result<(), String> {
    let mut engine_guard = state.engine.lock().await;
    let engine = engine_guard.as_mut().ok_or("engine not initialized")?;

    engine.manager.pause_task(&task_id).await;
    Ok(())
}

#[tauri::command]
pub async fn resume_task(state: State<'_, AppState>, task_id: String) -> Result<(), String> {
    let mut engine_guard = state.engine.lock().await;
    let engine = engine_guard.as_mut().ok_or("engine not initialized")?;

    engine.manager.resume_task(&task_id).await;
    Ok(())
}

#[tauri::command]
pub async fn set_task_priority(
    state: State<'_, AppState>,
    task_id: String,
) -> Result<(), String> {
    let mut engine_guard = state.engine.lock().await;
    let engine = engine_guard.as_mut().ok_or("engine not initialized")?;
    engine.manager.set_priority_task(task_id).await;
    Ok(())
}

#[tauri::command]
pub async fn move_task_to_queue(
    state: State<'_, AppState>,
    task_id: String,
    queue_id: String,
) -> Result<(), String> {
    let mut engine_guard = state.engine.lock().await;
    let engine = engine_guard.as_mut().ok_or("engine not initialized")?;
    engine.manager.move_task_to_queue(task_id, queue_id).await;
    Ok(())
}

#[tauri::command]
pub async fn reveal_in_folder(path: String) -> Result<(), String> {
    let p = std::path::Path::new(&path);
    if p.exists() {
        #[cfg(target_os = "macos")]
        std::process::Command::new("open")
            .arg("-R")
            .arg(&path)
            .spawn()
            .map_err(|e| e.to_string())?;
        #[cfg(target_os = "windows")]
        std::process::Command::new("explorer")
            .arg("/select,")
            .arg(&path)
            .spawn()
            .map_err(|e| e.to_string())?;
        #[cfg(target_os = "linux")]
        std::process::Command::new("xdg-open")
            .arg(p.parent().unwrap_or(p))
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub async fn send_notification(title: String, body: String) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        let script = format!(
            "display notification \"{}\" with title \"{}\"",
            body.replace('"', "\\\""),
            title.replace('"', "\\\"")
        );
        std::process::Command::new("osascript")
            .arg("-e")
            .arg(&script)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "windows")]
    {
        // Use PowerShell for Windows notifications
        let script = format!(
            "[Windows.UI.Notifications.ToastNotificationManager, Windows.UI.Notifications, ContentType = WindowsRuntime] > $null; \
             $template = [Windows.UI.Notifications.ToastNotificationManager]::GetTemplateContent([Windows.UI.Notifications.ToastTemplateType]::ToastText02); \
             $textNodes = $template.GetElementsByTagName('text'); \
             $textNodes.Item(0).AppendChild($template.CreateTextNode('{}')) > $null; \
             $textNodes.Item(1).AppendChild($template.CreateTextNode('{}')) > $null; \
             $toast = [Windows.UI.Notifications.ToastNotification]::new($template); \
             [Windows.UI.Notifications.ToastNotificationManager]::CreateToastNotifier('ns-download').Show($toast)",
            title, body
        );
        let _ = std::process::Command::new("powershell")
            .args(["-NoProfile", "-Command", &script])
            .spawn();
    }
    #[cfg(target_os = "linux")]
    {
        let _ = std::process::Command::new("notify-send")
            .args([&title, &body])
            .spawn();
    }
    Ok(())
}

#[tauri::command]
pub async fn prevent_sleep(prevent: bool) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        if prevent {
            std::process::Command::new("caffeinate")
                .args(["-dimsu", "-t", "86400"])
                .spawn()
                .map_err(|e| e.to_string())?;
        }
        // Note: Killing the caffeinate process is more complex
        // For simplicity, we just let it time out
    }
    #[cfg(target_os = "windows")]
    {
        if prevent {
            std::process::Command::new("powercfg")
                .args(["/change", "standby-timeout-ac", "0"])
                .spawn()
                .map_err(|e| e.to_string())?;
        }
    }
    #[cfg(target_os = "linux")]
    {
        if prevent {
            let _ = std::process::Command::new("systemd-inhibit")
                .args(["--what=sleep", "--who=ns-download", "--why=Downloading",
                       &std::env::current_exe().unwrap_or_default().to_string_lossy()])
                .spawn();
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn shutdown_system(action: String) -> Result<(), String> {
    match action.as_str() {
        "shutdown" => {
            #[cfg(target_os = "macos")]
            std::process::Command::new("osascript")
                .args(["-e", "tell app \"System Events\" to shut down"])
                .spawn().map_err(|e| e.to_string())?;
            #[cfg(target_os = "windows")]
            std::process::Command::new("shutdown")
                .args(["/s", "/f", "/t", "0"])
                .spawn().map_err(|e| e.to_string())?;
            #[cfg(target_os = "linux")]
            std::process::Command::new("shutdown")
                .args(["now"])
                .spawn().map_err(|e| e.to_string())?;
        }
        "sleep" => {
            #[cfg(target_os = "macos")]
            std::process::Command::new("osascript")
                .args(["-e", "tell app \"System Events\" to sleep"])
                .spawn().map_err(|e| e.to_string())?;
            #[cfg(target_os = "windows")]
            std::process::Command::new("rundll32.exe")
                .args(["powrprof.dll,SetSuspendState", "0", "1", "0"])
                .spawn().map_err(|e| e.to_string())?;
            #[cfg(target_os = "linux")]
            std::process::Command::new("systemctl")
                .args(["suspend"])
                .spawn().map_err(|e| e.to_string())?;
        }
        "hibernate" => {
            #[cfg(target_os = "macos")]
            std::process::Command::new("osascript")
                .args(["-e", "tell app \"System Events\" to sleep"])
                .spawn().map_err(|e| e.to_string())?;
            #[cfg(target_os = "windows")]
            std::process::Command::new("shutdown")
                .args(["/h", "/f"])
                .spawn().map_err(|e| e.to_string())?;
            #[cfg(target_os = "linux")]
            std::process::Command::new("systemctl")
                .args(["hibernate"])
                .spawn().map_err(|e| e.to_string())?;
        }
        _ => return Err("Unknown action, use: shutdown, sleep, hibernate".into()),
    }
    Ok(())
}

#[tauri::command]
pub async fn remove_task(
    state: State<'_, AppState>,
    task_id: String,
    delete_files: bool,
) -> Result<(), String> {
    let mut engine_guard = state.engine.lock().await;
    let engine = engine_guard.as_mut().ok_or("engine not initialized")?;

    engine
        .manager
        .delete_task(&task_id, delete_files)
        .await;
    Ok(())
}

#[derive(Serialize)]
pub struct ProxyTestResult {
    pub success: bool,
    pub latency_ms: i64,
    pub error_message: String,
}

#[tauri::command]
pub async fn test_proxy(
    state: State<'_, AppState>,
    proxy_type: String,
    proxy_host: String,
    proxy_port: String,
    proxy_username: String,
    proxy_password: String,
) -> Result<ProxyTestResult, String> {
    let engine = state.engine.lock().await;
    let engine = engine.as_ref().ok_or("engine not initialized")?;

    match engine
        .test_proxy_connection(&proxy_type, &proxy_host, &proxy_port, &proxy_username, &proxy_password)
        .await
    {
        Ok(latency) => Ok(ProxyTestResult {
            success: true,
            latency_ms: latency,
            error_message: String::new(),
        }),
        Err(e) => Ok(ProxyTestResult {
            success: false,
            latency_ms: 0,
            error_message: e.to_string(),
        }),
    }
}

#[tauri::command]
pub async fn load_settings(state: State<'_, AppState>) -> Result<HashMap<String, String>, String> {
    let engine_guard = state.engine.lock().await;
    let engine = engine_guard.as_ref().ok_or("engine not initialized")?;
    settings::load_all(engine).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn save_settings(
    state: State<'_, AppState>,
    settings: HashMap<String, String>,
) -> Result<(), String> {
    let mut engine_guard = state.engine.lock().await;
    let engine = engine_guard.as_mut().ok_or("engine not initialized")?;
    settings::save_all_and_apply(engine, &settings).await
}

#[derive(Serialize)]
pub struct CheckUpdateResult {
    pub has_update: bool,
    pub latest_version: String,
    pub download_url: String,
    pub error_message: String,
}

#[tauri::command]
pub async fn check_update(current_version: String) -> Result<CheckUpdateResult, String> {
    let url = format!(
        "https://api.github.com/repos/navysummer/ns-download/releases/latest"
    );
    let client = reqwest::Client::builder()
        .user_agent("ns-download")
        .build()
        .map_err(|e| e.to_string())?;

    match client.get(&url).send().await {
        Ok(resp) => {
            if let Ok(data) = resp.json::<serde_json::Value>().await {
                let latest = data["tag_name"].as_str().unwrap_or("").to_string();
                let has_update = !latest.is_empty() && latest != current_version;
                let download_url = data["html_url"].as_str().unwrap_or("").to_string();
                return Ok(CheckUpdateResult {
                    has_update,
                    latest_version: latest,
                    download_url,
                    error_message: String::new(),
                });
            }
            Ok(CheckUpdateResult {
                has_update: false,
                latest_version: String::new(),
                download_url: String::new(),
                error_message: "Failed to parse response".to_string(),
            })
        }
        Err(e) => Ok(CheckUpdateResult {
            has_update: false,
            latest_version: String::new(),
            download_url: String::new(),
            error_message: e.to_string(),
        }),
    }
}

#[tauri::command]
pub async fn check_command_exists(name: String) -> Result<Option<String>, String> {
    let output = std::process::Command::new("which")
        .arg(&name)
        .output()
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
        // Try to get version
        let version_output = std::process::Command::new(&name)
            .arg("--version")
            .output()
            .ok();
        if let Some(vo) = version_output {
            if vo.status.success() {
                let version_line = String::from_utf8_lossy(&vo.stdout)
                    .lines()
                    .next()
                    .unwrap_or("")
                    .to_string();
                return Ok(Some(format!("{} @ {}", version_line, path)));
            }
        }
        Ok(Some(path))
    } else {
        Ok(None)
    }
}

#[tauri::command]
pub async fn export_logs(
    state: State<'_, AppState>,
    dest_path: String,
) -> Result<(), String> {
    let engine_guard = state.engine.lock().await;
    let engine = engine_guard.as_ref().ok_or("engine not initialized")?;

    let data_dir = &engine.data_dir;
    let log_files = [
        data_dir.join("ns-download.log"),
        data_dir.join("engine.log"),
    ];

    let dest = std::path::PathBuf::from(&dest_path);
    if dest.is_dir() {
        for log_file in &log_files {
            if log_file.exists() {
                let file_name = log_file.file_name().unwrap();
                let dest_file = dest.join(file_name);
                std::fs::copy(log_file, &dest_file).map_err(|e| e.to_string())?;
            }
        }
    } else {
        // Copy to a single file
        if let Some(log_file) = log_files.iter().find(|f| f.exists()) {
            std::fs::copy(log_file, &dest).map_err(|e| e.to_string())?;
        } else {
            return Err("No log files found".to_string());
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn start_api_server(state: State<'_, AppState>) -> Result<(), String> {
    let _engine_guard = state.engine.lock().await;
    // The API server requires the native/api crate which is currently a stub.
    // TODO: implement when native/api is completed
    tracing::warn!("API server not yet implemented");
    Ok(())
}

#[tauri::command]
pub async fn stop_api_server(state: State<'_, AppState>) -> Result<(), String> {
    let _engine_guard = state.engine.lock().await;
    tracing::warn!("API server not yet implemented");
    Ok(())
}

fn dirs_or_fallback() -> String {
    std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map(|h| format!("{}/Downloads", h))
        .unwrap_or_else(|_| "/tmp/downloads".to_string())
}

fn app_data_dir() -> String {
    let d = dirs::data_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
    d.join("ns-download").to_string_lossy().to_string()
}
