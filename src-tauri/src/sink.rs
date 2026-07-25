use ns_download_engine::events::{EngineEvent, EventSink};
use ns_download_engine::model::TaskInfo;
use serde::Serialize;
use tauri::Emitter;

#[derive(Clone, Serialize)]
pub struct TaskProgressPayload {
    pub task_id: String,
    pub status: i32,
    pub downloaded_bytes: i64,
    pub total_bytes: i64,
    pub speed: i64,
    pub file_name: String,
    pub save_dir: String,
    pub url: String,
    pub error_message: String,
    pub upload_speed_bps: i64,
}

#[derive(Clone, Serialize)]
pub struct TasksSnapshotPayload {
    pub tasks: Vec<TaskResponsePayload>,
}

#[derive(Clone, Serialize)]
pub struct TaskResponsePayload {
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

impl From<TaskInfo> for TaskResponsePayload {
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

#[derive(Clone, Serialize)]
pub struct SegmentProgressPayload {
    pub task_id: String,
    pub total_bytes: i64,
    pub segment_count: i32,
    pub segments: Vec<SegmentDetailPayload>,
}

#[derive(Clone, Serialize)]
pub struct SegmentDetailPayload {
    pub index: i32,
    pub start_byte: i64,
    pub end_byte: i64,
    pub downloaded_bytes: i64,
}

/// Event sink that forwards engine events to the Tauri frontend.
pub struct TauriEventSink {
    app_handle: tauri::AppHandle,
}

impl TauriEventSink {
    pub fn new(app_handle: tauri::AppHandle) -> Self {
        Self { app_handle }
    }
}

impl EventSink for TauriEventSink {
    fn emit(&self, event: EngineEvent) {
        match event {
            EngineEvent::TaskProgress {
                task_id,
                status,
                downloaded_bytes,
                total_bytes,
                speed,
                file_name,
                save_dir,
                url,
                error_message,
                upload_speed_bps,
            } => {
                let payload = TaskProgressPayload {
                    task_id,
                    status,
                    downloaded_bytes,
                    total_bytes,
                    speed,
                    file_name,
                    save_dir,
                    url,
                    error_message,
                    upload_speed_bps,
                };
                let _ = self.app_handle.emit("task-progress", payload);
            }
            EngineEvent::TasksSnapshot(tasks) => {
                let tasks: Vec<TaskResponsePayload> = tasks.into_iter().map(Into::into).collect();
                let payload = TasksSnapshotPayload { tasks };
                let _ = self.app_handle.emit("tasks-snapshot", payload);
            }
            EngineEvent::SegmentProgress {
                task_id,
                total_bytes,
                segment_count,
                segments,
            } => {
                let segments: Vec<SegmentDetailPayload> = segments
                    .into_iter()
                    .map(|s| SegmentDetailPayload {
                        index: s.index,
                        start_byte: s.start_byte,
                        end_byte: s.end_byte,
                        downloaded_bytes: s.downloaded_bytes,
                    })
                    .collect();
                let payload = SegmentProgressPayload {
                    task_id,
                    total_bytes,
                    segment_count,
                    segments,
                };
                let _ = self.app_handle.emit("segment-progress", payload);
            }
            EngineEvent::TaskMetaProbed {
                task_id,
                file_name,
                total_bytes,
            } => {
                let payload = serde_json::json!({
                    "task_id": task_id,
                    "file_name": file_name,
                    "total_bytes": total_bytes,
                });
                let _ = self.app_handle.emit("task-meta-probed", payload);
            }
            EngineEvent::BtDataFinished { task_id } => {
                let payload = serde_json::json!({ "task_id": task_id });
                let _ = self.app_handle.emit("bt-data-finished", payload);
            }
            EngineEvent::QueuePositionsChanged(positions) => {
                let positions: Vec<serde_json::Value> = positions
                    .into_iter()
                    .map(|p| {
                        serde_json::json!({
                            "task_id": p.task_id,
                            "position": p.position,
                        })
                    })
                    .collect();
                let _ = self.app_handle.emit("queue-positions-changed", positions);
            }
            EngineEvent::QueuesChanged(queues) => {
                let queues: Vec<serde_json::Value> = queues
                    .into_iter()
                    .map(|q| {
                        serde_json::json!({
                            "queue_id": q.queue_id,
                            "name": q.name,
                            "speed_limit_kbps": q.speed_limit_kbps,
                            "max_concurrent": q.max_concurrent,
                            "default_save_dir": q.default_save_dir,
                            "is_running": q.is_running,
                        })
                    })
                    .collect();
                let _ = self.app_handle.emit("queues-changed", queues);
            }
            EngineEvent::TaskQueueChanged { task_id, queue_id } => {
                let payload = serde_json::json!({ "task_id": task_id, "queue_id": queue_id });
                let _ = self.app_handle.emit("task-queue-changed", payload);
            }
            EngineEvent::PriorityTaskChanged {
                priority_task_id,
                auto_paused_count,
            } => {
                let payload = serde_json::json!({
                    "priority_task_id": priority_task_id,
                    "auto_paused_count": auto_paused_count,
                });
                let _ = self.app_handle.emit("priority-task-changed", payload);
            }
            _ => {}
        }
    }
}
