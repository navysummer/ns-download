use crate::model::{QueueInfo, QueuePosition, SegmentDetail, TaskInfo};

#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum EngineEvent {
    TaskProgress {
        task_id: String,
        status: i32,
        downloaded_bytes: i64,
        total_bytes: i64,
        speed: i64,
        file_name: String,
        save_dir: String,
        url: String,
        error_message: String,
        upload_speed_bps: i64,
    },
    BtDataFinished { task_id: String },
    TasksSnapshot(Vec<TaskInfo>),
    SegmentProgress {
        task_id: String,
        total_bytes: i64,
        segment_count: i32,
        segments: Vec<SegmentDetail>,
    },
    TaskMetaProbed {
        task_id: String,
        file_name: String,
        total_bytes: i64,
    },
    QueuePositionsChanged(Vec<QueuePosition>),
    QueuesChanged(Vec<QueueInfo>),
    TaskQueueChanged { task_id: String, queue_id: String },
    PriorityTaskChanged {
        priority_task_id: String,
        auto_paused_count: i32,
    },
    SegmentSplit {
        task_id: String,
        parent_index: i32,
        parent_new_end: i64,
        child_index: i32,
        child_start: i64,
        child_end: i64,
        is_proactive: bool,
        total_segments: i32,
    },
    FileMissingChanged(Vec<(String, bool)>),
    PluginAutoDisabled {
        identity: String,
        reason: String,
    },
    PluginHookActivity {
        task_id: String,
        plugin_id: String,
        running: bool,
    },
}

pub trait EventSink: Send + Sync {
    fn emit(&self, event: EngineEvent);
}
