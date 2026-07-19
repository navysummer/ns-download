pub const MAIN_QUEUE_ID: &str = "main";
pub const LATER_QUEUE_ID: &str = "later";

pub fn is_builtin_queue(queue_id: &str) -> bool {
    queue_id == MAIN_QUEUE_ID || queue_id == LATER_QUEUE_ID
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskInfo {
    pub task_id: String,
    pub url: String,
    pub file_name: String,
    pub save_dir: String,
    pub status: i32,
    pub downloaded_bytes: i64,
    pub total_bytes: i64,
    pub error_message: String,
    pub created_at: String,
    pub proxy_url: String,
    pub queue_id: String,
    pub checksum: String,
    pub file_missing: bool,
    pub completed_at: String,
    pub segments: i32,
    pub queue_order: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueueInfo {
    pub queue_id: String,
    pub name: String,
    pub speed_limit_kbps: i64,
    pub max_concurrent: i32,
    pub default_save_dir: String,
    pub position: i32,
    pub default_segments: i32,
    pub default_user_agent: String,
    pub is_running: bool,
    pub schedule_enabled: bool,
    pub schedule_start: String,
    pub schedule_stop: String,
    pub schedule_days: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueuePosition {
    pub task_id: String,
    pub position: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SegmentDetail {
    pub index: i32,
    pub start_byte: i64,
    pub end_byte: i64,
    pub downloaded_bytes: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BtFileEntry {
    pub index: i32,
    pub path: String,
    pub size: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HlsQualityOption {
    pub index: i32,
    pub bandwidth: i64,
    pub width: i64,
    pub height: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolveVariantOption {
    pub index: i32,
    pub label: String,
    pub container: String,
    pub bandwidth: i64,
    pub width: i64,
    pub height: i64,
    pub total_bytes: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TorrentMetaResult {
    pub probe_id: String,
    pub name: String,
    pub total_bytes: i64,
    pub files: Vec<BtFileEntry>,
    pub error: String,
}
