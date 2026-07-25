use std::sync::Arc;
use std::net::SocketAddr;
use axum::{
    Router,
    extract::{State, Path},
    http::{StatusCode, HeaderMap},
    response::{IntoResponse, Response, Json},
    routing::{get, post, put, delete},
    middleware,
};
use tokio::sync::Mutex;
use ns_download_engine::Engine;
use ns_download_engine::download_manager::NewTaskSpec;

type EngineRef = Arc<Mutex<Option<Engine>>>;

pub enum ServerCommand {
    Stop,
}

struct ServerState {
    engine: EngineRef,
    token: String,
}

fn check_auth(headers: &HeaderMap, token: &str) -> bool {
    if token.is_empty() { return true; }
    headers.get("Authorization")
        .and_then(|v| v.to_str().ok())
        .map(|v| v == token || v.strip_prefix("Bearer ").map_or(false, |t| t == token))
        .unwrap_or(false)
}

async fn auth_middleware(
    State(srv): State<Arc<ServerState>>,
    headers: HeaderMap,
    req: axum::extract::Request,
    next: middleware::Next,
) -> Response {
    if !check_auth(&headers, &srv.token) {
        return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response();
    }
    next.run(req).await
}

async fn handle_takeover() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "message": "ns-download API server is running",
        "endpoints": {
            "takeover": "/ (POST with url parameter)",
            "jsonrpc": "/jsonrpc",
            "api_v1": "/api/v1/...",
            "mcp": "/mcp"
        }
    }))
}

async fn handle_takeover_post(
    State(srv): State<Arc<ServerState>>,
    Json(body): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    let url = body.get("url").and_then(|u| u.as_str()).unwrap_or("");
    if url.is_empty() {
        return Json(serde_json::json!({"status": "error", "message": "missing url parameter"}));
    }
    let mut engine = srv.engine.lock().await;
    match engine.as_mut() {
        Some(eng) => {
            let spec = NewTaskSpec {
                url: url.to_string(),
                save_dir: String::new(),
                file_name: String::new(),
                segments: 0,
                cookies: String::new(),
                referrer: String::new(),
                hint_file_size: 0,
                torrent_file_bytes: Vec::new(),
                proxy_url: String::new(),
                user_agent: String::new(),
                queue_id: String::new(),
                checksum: String::new(),
                extra_headers: std::collections::HashMap::new(),
                selected_file_indices: Vec::new(),
                method: None,
                body: None,
                audio_url: None,
                start_paused: false,
                overwrite: false,
            };
            match eng.manager.create_task(spec).await {
                Some(task_id) => Json(serde_json::json!({"status": "ok", "task_id": task_id})),
                None => Json(serde_json::json!({"status": "error", "message": "failed to create task"})),
            }
        }
        None => Json(serde_json::json!({"status": "error", "message": "engine not initialized"})),
    }
}

async fn handle_jsonrpc(
    State(srv): State<Arc<ServerState>>,
    Json(body): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    let method = body.get("method").and_then(|m| m.as_str()).unwrap_or("");
    let id = body.get("id");

    let result = match method {
        "list_tasks" | "get_tasks" => {
            let engine = srv.engine.lock().await;
            if let Some(ref eng) = *engine {
                let tasks = eng.db.load_all_tasks().await.unwrap_or_default();
                let task_list: Vec<serde_json::Value> = tasks.iter().map(|t| {
                    serde_json::json!({
                        "id": t.task_id,
                        "url": t.url,
                        "file_name": t.file_name,
                        "save_dir": t.save_dir,
                        "status": t.status,
                        "downloaded_bytes": t.downloaded_bytes,
                        "total_bytes": t.total_bytes,
                        "error_message": t.error_message,
                        "created_at": t.created_at,
                        "completed_at": t.completed_at,
                    })
                }).collect();
                serde_json::json!(task_list)
            } else {
                serde_json::json!({"error": "engine not initialized"})
            }
        }
        "get_version" => {
            serde_json::json!({"version": "0.1.0", "name": "ns-download"})
        }
        _ => serde_json::json!({"error": format!("unknown method: {}", method)}),
    };

    let mut response = serde_json::json!({"jsonrpc": "2.0", "result": result});
    if let Some(id) = id {
        response["id"] = id.clone();
    }
    Json(response)
}

async fn handle_mcp(
    State(srv): State<Arc<ServerState>>,
    Json(body): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    let method = body.get("method").and_then(|m| m.as_str()).unwrap_or("");
    match method {
        "list_resources" => {
            let engine = srv.engine.lock().await;
            if let Some(ref eng) = *engine {
                let tasks = eng.db.load_all_tasks().await.unwrap_or_default();
                let resources: Vec<serde_json::Value> = tasks.iter().map(|t| {
                    serde_json::json!({
                        "uri": format!("ns-download://task/{}", t.task_id),
                        "name": t.file_name,
                        "mimeType": "application/octet-stream",
                    })
                }).collect();
                Json(serde_json::json!({"resources": resources}))
            } else {
                Json(serde_json::json!({"resources": []}))
            }
        }
        _ => Json(serde_json::json!({"error": format!("unknown method: {}", method)})),
    }
}

async fn list_tasks(
    State(srv): State<Arc<ServerState>>,
) -> Json<serde_json::Value> {
    let engine = srv.engine.lock().await;
    if let Some(ref eng) = *engine {
        let tasks = eng.db.load_all_tasks().await.unwrap_or_default();
        let task_list: Vec<serde_json::Value> = tasks.iter().map(|t| {
            serde_json::json!({
                "id": t.task_id,
                "url": t.url,
                "file_name": t.file_name,
                "save_dir": t.save_dir,
                "status": t.status,
                "downloaded_bytes": t.downloaded_bytes,
                "total_bytes": t.total_bytes,
                "error_message": t.error_message,
                "created_at": t.created_at,
                "completed_at": t.completed_at,
                "segments": t.segments,
                "queue_id": t.queue_id,
            })
        }).collect();
        Json(serde_json::json!({"tasks": task_list}))
    } else {
        Json(serde_json::json!({"error": "engine not initialized", "tasks": []}))
    }
}

async fn create_task_api(
    State(srv): State<Arc<ServerState>>,
    Json(body): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    let url = body.get("url").and_then(|u| u.as_str()).unwrap_or("");
    if url.is_empty() {
        return Json(serde_json::json!({"status": "error", "message": "missing url parameter"}));
    }
    let mut engine = srv.engine.lock().await;
    match engine.as_mut() {
        Some(eng) => {
            let spec = NewTaskSpec {
                url: url.to_string(),
                save_dir: body.get("save_dir").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                file_name: body.get("file_name").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                segments: body.get("segments").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
                cookies: body.get("cookies").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                referrer: body.get("referrer").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                hint_file_size: body.get("hint_file_size").and_then(|v| v.as_i64()).unwrap_or(0),
                torrent_file_bytes: Vec::new(),
                proxy_url: body.get("proxy_url").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                user_agent: body.get("user_agent").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                queue_id: body.get("queue_id").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                checksum: body.get("checksum").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                extra_headers: std::collections::HashMap::new(),
                selected_file_indices: Vec::new(),
                method: None,
                body: None,
                audio_url: None,
                start_paused: body.get("start_paused").and_then(|v| v.as_bool()).unwrap_or(false),
                overwrite: body.get("overwrite").and_then(|v| v.as_bool()).unwrap_or(false),
            };
            match eng.manager.create_task(spec).await {
                Some(task_id) => Json(serde_json::json!({"status": "ok", "task_id": task_id})),
                None => Json(serde_json::json!({"status": "error", "message": "failed to create task"})),
            }
        }
        None => Json(serde_json::json!({"status": "error", "message": "engine not initialized"})),
    }
}

async fn remove_task_api(
    State(srv): State<Arc<ServerState>>,
    Path(id): Path<String>,
) -> Json<serde_json::Value> {
    let mut engine = srv.engine.lock().await;
    match engine.as_mut() {
        Some(eng) => {
            eng.manager.delete_task(&id, false).await;
            Json(serde_json::json!({"status": "ok"}))
        }
        None => Json(serde_json::json!({"status": "error", "message": "engine not initialized"})),
    }
}

async fn pause_task_api(
    State(srv): State<Arc<ServerState>>,
    Path(id): Path<String>,
) -> Json<serde_json::Value> {
    let mut engine = srv.engine.lock().await;
    match engine.as_mut() {
        Some(eng) => {
            eng.manager.pause_task(&id).await;
            Json(serde_json::json!({"status": "ok"}))
        }
        None => Json(serde_json::json!({"status": "error", "message": "engine not initialized"})),
    }
}

async fn resume_task_api(
    State(srv): State<Arc<ServerState>>,
    Path(id): Path<String>,
) -> Json<serde_json::Value> {
    let mut engine = srv.engine.lock().await;
    match engine.as_mut() {
        Some(eng) => {
            eng.manager.resume_task(&id).await;
            Json(serde_json::json!({"status": "ok"}))
        }
        None => Json(serde_json::json!({"status": "error", "message": "engine not initialized"})),
    }
}

async fn shutdown_signal(mut rx: tokio::sync::mpsc::Receiver<ServerCommand>) {
    rx.recv().await;
}

pub async fn run_server(
    engine: EngineRef,
    port: u16,
    token: String,
    shutdown_rx: tokio::sync::mpsc::Receiver<ServerCommand>,
    takeover_enabled: bool,
    jsonrpc_enabled: bool,
    api_enabled: bool,
    mcp_enabled: bool,
) -> Result<(), String> {
    let srv_state = Arc::new(ServerState { engine, token });

    let mut router = Router::new();
    if takeover_enabled {
        router = router.route("/", get(handle_takeover).post(handle_takeover_post));
    }
    if jsonrpc_enabled {
        router = router.route("/jsonrpc", post(handle_jsonrpc));
    }
    if api_enabled {
        router = router.route("/api/v1/tasks", get(list_tasks).post(create_task_api));
        router = router.route("/api/v1/tasks/{id}", delete(remove_task_api));
        router = router.route("/api/v1/tasks/{id}/pause", put(pause_task_api));
        router = router.route("/api/v1/tasks/{id}/resume", put(resume_task_api));
    }
    if mcp_enabled {
        router = router.route("/mcp", post(handle_mcp));
    }
    let app = router
        .layer(middleware::from_fn_with_state(srv_state.clone(), auth_middleware))
        .with_state(srv_state);

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    tracing::info!("Starting API server on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.map_err(|e| e.to_string())?;
    axum::serve(listener, app)
        .with_graceful_shutdown(async { shutdown_signal(shutdown_rx).await })
        .await
        .map_err(|e| e.to_string())?;

    tracing::info!("API server stopped");
    Ok(())
}
