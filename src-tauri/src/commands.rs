use std::sync::Arc;
use tauri::State;
use ns_download_engine::{Engine, EngineConfig, NoopSink, NoopSelection};
use ns_download_engine::bt_downloader::BtConfig;
use ns_download_engine::proxy_config::ProxyConfig;
use ns_download_engine::model::TaskInfo;
use serde::{Deserialize, Serialize};
use crate::AppState;

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
pub async fn init_engine(state: State<'_, AppState>) -> Result<(), String> {
    let mut engine_guard = state.engine.lock().await;
    if engine_guard.is_some() {
        return Ok(());
    }

    let config = EngineConfig {
        max_concurrent: 5,
        speed_limit_bps: 0,
        default_save_dir: dirs_or_fallback(),
        app_data_dir: app_data_dir(),
        bt_config: BtConfig::default(),
        proxy_config: ProxyConfig::default(),
        user_agent: String::new(),
        data_dir_override: None,
        database_url: None,
    };

    let engine = Engine::new(config, Arc::new(NoopSink), Arc::new(NoopSelection))
        .await
        .map_err(|e| e.to_string())?;

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
