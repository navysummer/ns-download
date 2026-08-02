use std::sync::Arc;
use std::collections::HashMap;
use std::time::Duration;
use tauri::State;
use ns_download_engine::{Engine, EngineConfig, NoopSelection};
use ns_download_engine::bt_downloader::BtConfig;
use ns_download_engine::proxy_config::ProxyConfig;
use ns_download_engine::model::TaskInfo;
use serde::{Deserialize, Serialize};
use base64::Engine as _;
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
    pub queue_id: String,
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
            queue_id: t.queue_id,
        }
    }
}

#[derive(Deserialize)]
pub struct CreateTaskSpec {
    pub url: String,
    pub save_dir: String,
    pub file_name: Option<String>,
    pub segments: Option<i32>,
    #[serde(default)]
    pub torrent_file_bytes: Vec<u8>,
    #[serde(default)]
    pub selected_file_indices: Vec<i32>,
    #[serde(default)]
    pub proxy_url: String,
    #[serde(default)]
    pub user_agent: String,
    #[serde(default)]
    pub cookies: String,
    #[serde(default)]
    pub referrer: String,
    #[serde(default)]
    pub checksum: String,
    #[serde(default)]
    pub extra_headers: std::collections::HashMap<String, String>,
    #[serde(default)]
    pub overwrite: bool,
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
    let sink_for_progress = sink.clone();

    let mut engine = Engine::new(config, sink, Arc::new(NoopSelection))
        .await
        .map_err(|e| e.to_string())?;

    // Initialize default config values in DB if not already set.
    engine.db.init_default_config(&default_save).await.map_err(|e| e.to_string())?;

    // Apply all saved settings from DB to the running engine.
    settings::apply_all(&mut engine).await;

    // Take the progress and done receivers so they are actually consumed.
    // Without this, progress channels fill up (buffer=64) and downloaders
    // block forever on send, making tasks appear stuck in "preparing" state.
    let progress_rx = engine.manager.take_progress_rx();
    let done_rx = engine.manager.take_done_rx();
    let retry_rx = engine.manager.take_retry_rx();
    let engine_for_done = state.engine.clone();
    let engine_for_retry = state.engine.clone();
    let db = engine.db.clone();

    *engine_guard = Some(engine);
    drop(engine_guard);

    // Spawn the progress reporter so progress updates are processed
    // instead of blocking the downloaders when the channel buffer fills up.
    if let Some(rx) = progress_rx {
        tokio::spawn(async move {
            ns_download_engine::download_manager::progress_reporter(rx, db, sink_for_progress).await;
        });
    }

    // Spawn the done handler so TaskDone messages are processed and
    // active_tasks entries are freed, allowing queued tasks to start.
    if let Some(mut rx) = done_rx {
        tokio::spawn(async move {
            while let Some(done) = rx.recv().await {
                let mut guard = engine_for_done.lock().await;
                if let Some(ref mut eng) = *guard {
                    eng.manager.on_task_done(&done).await;
                } else {
                    break;
                }
            }
        });
    }

    // Spawn the retry handler so auto-retry actually resumes tasks.
    if let Some(mut rx) = retry_rx {
        tokio::spawn(async move {
            while let Some(task_id) = rx.recv().await {
                let mut guard = engine_for_retry.lock().await;
                if let Some(ref mut eng) = *guard {
                    eng.manager.resume_task_auto(&task_id).await;
                } else {
                    break;
                }
            }
        });
    }

    spawn_subscription_tasks(state.engine.clone()).await;

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
        file_name: match spec.file_name {
            Some(ref n) if !n.is_empty() => n.clone(),
            _ => Default::default(),
        },
        segments: spec.segments.unwrap_or(0),
        torrent_file_bytes: spec.torrent_file_bytes,
        selected_file_indices: spec.selected_file_indices,
        proxy_url: spec.proxy_url,
        user_agent: spec.user_agent,
        cookies: spec.cookies,
        referrer: spec.referrer,
        checksum: spec.checksum,
        extra_headers: spec.extra_headers,
        overwrite: spec.overwrite,
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
pub async fn set_task_segments(
    state: State<'_, AppState>,
    task_id: String,
    segments: i32,
) -> Result<(), String> {
    let mut engine_guard = state.engine.lock().await;
    let engine = engine_guard.as_mut().ok_or("engine not initialized")?;
    engine.manager.set_task_segments(&task_id, segments).await
        .map_err(|e| e.to_string())?;
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

#[derive(Serialize, Deserialize)]
pub struct QueueDef {
    pub id: String,
    pub label: String,
    pub running: bool,
}

#[tauri::command]
pub async fn save_queues(
    state: State<'_, AppState>,
    queues: Vec<QueueDef>,
) -> Result<(), String> {
    let mut engine_guard = state.engine.lock().await;
    let engine = engine_guard.as_mut().ok_or("engine not initialized")?;
    let json = serde_json::to_string(&queues).map_err(|e| e.to_string())?;
    engine.db.set_config("queues", &json).await.map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn load_queues(
    state: State<'_, AppState>,
) -> Result<Vec<QueueDef>, String> {
    let engine_guard = state.engine.lock().await;
    let engine = engine_guard.as_ref().ok_or("engine not initialized")?;
    let raw = engine.db.get_config("queues").await.map_err(|e| e.to_string())?;
    match raw {
        Some(json) => serde_json::from_str(&json).map_err(|e| e.to_string()),
        None => Ok(vec![
            QueueDef { id: "default".into(), label: "默认".into(), running: true },
            QueueDef { id: "later".into(), label: "稍后下载".into(), running: false },
        ]),
    }
}

#[derive(Serialize)]
pub struct TorrentProbeFileResponse {
    pub index: i32,
    pub path: String,
    pub size: i64,
}

#[derive(Serialize)]
pub struct TorrentProbeResponse {
    pub name: String,
    pub total_bytes: i64,
    pub files: Vec<TorrentProbeFileResponse>,
    pub error: String,
}

#[tauri::command]
pub async fn probe_torrent_file(
    state: State<'_, AppState>,
    torrent_bytes: Vec<u8>,
) -> Result<TorrentProbeResponse, String> {
    let mut engine_guard = state.engine.lock().await;
    let engine = engine_guard.as_mut().ok_or("engine not initialized")?;
    let probe_id = uuid::Uuid::new_v4().to_string();
    let result = engine.probe_torrent_meta(probe_id, torrent_bytes).await;
    if !result.error.is_empty() {
        return Err(result.error);
    }
    Ok(TorrentProbeResponse {
        name: result.name,
        total_bytes: result.total_bytes,
        files: result.files.into_iter().map(|f| TorrentProbeFileResponse {
            index: f.index,
            path: f.path,
            size: f.size,
        }).collect(),
        error: result.error,
    })
}

#[tauri::command]
pub async fn reveal_in_folder(path: String) -> Result<(), String> {
    #[cfg(not(mobile))]
    {
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
            {
                if let Some(parent) = p.parent() {
                    std::process::Command::new("xdg-open")
                        .arg(parent)
                        .spawn()
                        .map_err(|e| e.to_string())?;
                }
            }
        }
        Ok(())
    }
    #[cfg(mobile)]
    {
        let _ = path;
        Err("Not supported on mobile".to_string())
    }
}

#[tauri::command]
pub async fn send_notification(title: String, body: String) -> Result<(), String> {
    #[cfg(not(mobile))]
    {
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
    #[cfg(mobile)]
    {
        let _ = (title, body);
        Err("Not supported on mobile".to_string())
    }
}

#[tauri::command]
pub async fn prevent_sleep(prevent: bool) -> Result<(), String> {
    #[cfg(not(mobile))]
    {
        #[cfg(target_os = "macos")]
        {
            use std::process::{Command, Stdio};
            if prevent {
                Command::new("caffeinate")
                    .args(["-dimsu", "-t", "86400"])
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .spawn()
                    .map_err(|e| e.to_string())?;
            } else {
                let _ = Command::new("pkill")
                    .args(["-f", "caffeinate -dimsu"])
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .spawn();
            }
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
    #[cfg(mobile)]
    {
        let _ = prevent;
        Err("Not supported on mobile".to_string())
    }
}

#[tauri::command]
pub async fn shutdown_system(action: String) -> Result<(), String> {
    #[cfg(mobile)]
    {
        let _ = action;
        return Err("Not supported on mobile".to_string());
    }

    #[cfg(not(mobile))]
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

    #[cfg(not(mobile))]
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
    pub body: String,
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
                let body = data["body"].as_str().unwrap_or("").to_string();
                return Ok(CheckUpdateResult {
                    has_update,
                    latest_version: latest,
                    download_url,
                    body,
                    error_message: String::new(),
                });
            }
            Ok(CheckUpdateResult {
                has_update: false,
                latest_version: String::new(),
                download_url: String::new(),
                body: String::new(),
                error_message: "Failed to parse response".to_string(),
            })
        }
        Err(e) => Ok(CheckUpdateResult {
            has_update: false,
            latest_version: String::new(),
            download_url: String::new(),
            body: String::new(),
            error_message: e.to_string(),
        }),
    }
}

#[tauri::command]
pub async fn check_command_exists(name: String) -> Result<Option<String>, String> {
    #[cfg(mobile)]
    {
        let _ = name;
        return Ok(None);
    }

    #[cfg(not(mobile))]
    {
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
    let (tx, rx) = tokio::sync::mpsc::channel(1);

    let engine = state.engine.clone();
    let (port, token, takeover_enabled, jsonrpc_enabled, api_enabled, mcp_enabled) = {
        let engine_guard = state.engine.lock().await;
        let eng = engine_guard.as_ref().ok_or("engine not initialized")?;
        let port_str = eng.db.get_config("local_server_port").await
            .ok().flatten().unwrap_or_else(|| "17800".to_string());
        let token = eng.db.get_config("local_server_token").await
            .ok().flatten().unwrap_or_default();
        let port: u16 = port_str.parse().unwrap_or(17800);
        let takeover_enabled = eng.db.get_config("local_server_takeover_enabled").await
            .ok().flatten().map(|v| v == "true").unwrap_or(true);
        let jsonrpc_enabled = eng.db.get_config("local_server_jsonrpc_enabled").await
            .ok().flatten().map(|v| v == "true" || v == "true").unwrap_or(true);
        let api_enabled = eng.db.get_config("local_server_api_enabled").await
            .ok().flatten().map(|v| v == "true").unwrap_or(true);
        let mcp_enabled = eng.db.get_config("local_server_mcp_enabled").await
            .ok().flatten().map(|v| v == "true").unwrap_or(true);
        // Store the shutdown sender
        let mut shutdown_guard = state.api_server_shutdown.lock().await;
        *shutdown_guard = Some(tx);
        (port, token, takeover_enabled, jsonrpc_enabled, api_enabled, mcp_enabled)
    };

    // Spawn server in background
    tokio::spawn(async move {
        let result = crate::api_server::run_server(engine, port, token, rx, takeover_enabled, jsonrpc_enabled, api_enabled, mcp_enabled).await;
        if let Err(e) = result {
            tracing::error!("API server failed: {e}");
        }
    });

    tracing::info!("API server started on port {}", port);
    Ok(())
}

#[tauri::command]
pub async fn stop_api_server(state: State<'_, AppState>) -> Result<(), String> {
    let mut shutdown_guard = state.api_server_shutdown.lock().await;
    if let Some(tx) = shutdown_guard.take() {
        let _ = tx.send(crate::api_server::ServerCommand::Stop).await;
        tracing::info!("API server stop signal sent");
    }
    Ok(())
}

/// Spawn background tasks that periodically refresh BT tracker and ED2K server
/// subscriptions from community URLs.
async fn spawn_subscription_tasks(engine_ref: Arc<tokio::sync::Mutex<Option<Engine>>>) {
    // BT tracker subscription refresh
    tokio::spawn({
        let engine_ref = engine_ref.clone();
        async move {
            loop {
                let (sub_urls, sub_enabled) = {
                    let guard = engine_ref.lock().await;
                    if let Some(ref eng) = *guard {
                        let urls = eng.db.get_config("bt_tracker_sub_urls").await
                            .ok().flatten().unwrap_or_else(ns_download_engine::tracker_subscription::default_subscription_urls);
                        let enabled = eng.db.get_config("bt_tracker_sub_enabled").await
                            .ok().flatten().map(|v| v == "true").unwrap_or(true);
                        (urls, enabled)
                    } else {
                        break;
                    }
                };

                if sub_enabled && !sub_urls.is_empty() {
                    tracing::info!("Refreshing BT tracker subscriptions...");
                    let outcome = ns_download_engine::tracker_subscription::fetch_subscriptions(&sub_urls).await;
                    if outcome.is_success() {
                        let merged = outcome.trackers.join("\n");
                        let mut guard = engine_ref.lock().await;
                        if let Some(ref mut eng_bt) = *guard {
                            let _ = eng_bt.db.set_config("bt_tracker_sub_cache", &merged).await;
                            settings::apply_bt_config(eng_bt, "bt_tracker_sub_cache", &merged).await;
                        }
                    }
                }

                tokio::time::sleep(Duration::from_secs(
                    ns_download_engine::tracker_subscription::REFRESH_INTERVAL_SECS as u64
                )).await;
            }
        }
    });

     // ED2K server subscription refresh
     tokio::spawn({
         let engine_ref = engine_ref.clone();
         async move {
             loop {
                 let (sub_urls, sub_enabled) = {
                     let guard = engine_ref.lock().await;
                     if let Some(ref eng) = *guard {
                    let urls = eng.db.get_config("ed2k_server_sub_urls").await
                        .ok().flatten().unwrap_or_else(ns_download_engine::ed2k::server_subscription::default_server_met_urls);
                    let enabled = eng.db.get_config("ed2k_server_sub_enabled").await
                        .ok().flatten().map(|v| v == "true").unwrap_or(true);
                    (urls, enabled)
                } else {
                    break;
                }
            };

            if sub_enabled && !sub_urls.is_empty() {
                tracing::info!("Refreshing ED2K server subscriptions...");
                let outcome = ns_download_engine::ed2k::server_subscription::fetch_server_subscriptions(
                    &sub_urls,
                ).await;
                if outcome.is_success() {
                    let merged = outcome.servers.join("\n");
                    let guard = engine_ref.lock().await;
                    if let Some(ref eng) = *guard {
                        let _ = eng.db.set_config("ed2k_server_sub_cache", &merged).await;
                    }
                }
            }

                 tokio::time::sleep(Duration::from_secs(
                     ns_download_engine::ed2k::server_subscription::REFRESH_INTERVAL_SECS as u64
                 )).await;
             }
         }
     });

     // Kad nodes.dat auto-fetch (keep bootstrap contacts fresh so Kad
     // can find sources even when all eD2K servers are empty/no seeders).
     tokio::spawn({
         let engine_ref = engine_ref.clone();
         async move {
             loop {
                 // Read the nodes.dat URL each cycle (it may change via settings).
                 let kad_url = {
                     let guard = engine_ref.lock().await;
                     match guard.as_ref() {
                         Some(eng) => eng.db.get_config("ed2k_nodes_dat_url").await
                             .ok().flatten()
                             .unwrap_or_else(|| ns_download_engine::ed2k::kad::DEFAULT_NODES_DAT_URL.to_string()),
                         None => break,
                     }
                 };

                 // Respect Kad toggle each cycle.
                 let kad_enabled = {
                     let guard = engine_ref.lock().await;
                     match guard.as_ref() {
                         Some(eng) => eng.db.get_config("ed2k_enable_kad").await
                             .ok().flatten().map(|v| v == "true").unwrap_or(true),
                         None => break,
                     }
                 };

                 if kad_enabled {
                     tracing::info!("Refreshing Kad nodes.dat...");
                     match ns_download_engine::ed2k::kad::fetch_nodes_dat(&kad_url).await {
                         Ok(bytes) => {
                             let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
                             let guard = engine_ref.lock().await;
                             if let Some(ref eng) = *guard {
                                 let _ = eng.db.set_config("ed2k_nodes_dat_cache", &b64).await;
                             }
                         }
                         Err(e) => {
                             tracing::warn!("[ed2k-kad] nodes.dat fetch failed: {e}");
                         }
                     }
                 }

                 tokio::time::sleep(Duration::from_secs(
                     ns_download_engine::ed2k::kad::FETCH_INTERVAL_SECS
                 )).await;
             }
         }
     });
 }

fn dirs_or_fallback() -> String {
    dirs::download_dir()
        .or_else(|| dirs::home_dir())
        .map(|p| p.join("Downloads").to_string_lossy().to_string())
        .unwrap_or_else(|| "/tmp/downloads".to_string())
}

fn app_data_dir() -> String {
    let d = dirs::data_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
    d.join("ns-download").to_string_lossy().to_string()
}

#[derive(Serialize, Deserialize)]
pub struct FeedbackPayload {
    pub feedback_type: String,
    pub message: String,
    pub contact: String,
    pub time: String,
}

#[tauri::command]
pub async fn submit_feedback(feedback: FeedbackPayload) -> Result<(), String> {
    let dir = app_data_dir();
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let path = std::path::PathBuf::from(&dir).join("feedback.jsonl");
    let line = serde_json::to_string(&feedback).map_err(|e| e.to_string())?;
    let mut file = std::fs::OpenOptions::new()
        .create(true).append(true).open(&path)
        .map_err(|e| e.to_string())?;
    use std::io::Write;
    writeln!(file, "{}", line).map_err(|e| e.to_string())?;
    tracing::info!("Feedback saved: {}", line);
    Ok(())
}
