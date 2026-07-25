use std::sync::Arc;
use std::net::SocketAddr;
use axum::{
    Router,
    extract::State,
    http::{StatusCode, HeaderMap},
    response::{IntoResponse, Response, Json},
    routing::{get, post},
    middleware,
};
use tokio::sync::Mutex;
use ns_download_engine::Engine;

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

async fn shutdown_signal(mut rx: tokio::sync::mpsc::Receiver<ServerCommand>) {
    rx.recv().await;
}

pub async fn run_server(
    engine: EngineRef,
    port: u16,
    token: String,
    shutdown_rx: tokio::sync::mpsc::Receiver<ServerCommand>,
) -> Result<(), String> {
    let srv_state = Arc::new(ServerState { engine, token });

    let app = Router::new()
        .route("/", get(handle_takeover).post(handle_takeover))
        .route("/jsonrpc", post(handle_jsonrpc))
        .route("/mcp", post(handle_mcp))
        .route("/api/v1/tasks", get(list_tasks))
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
