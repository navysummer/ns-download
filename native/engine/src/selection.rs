use std::time::Duration;
use crate::model::{BtFileEntry, HlsQualityOption, ResolveVariantOption};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SelectionOutcome<T> {
    UserChose(T),
    TimedOutDefaulted(T),
    NoSelectorConfigured(T),
}

impl<T> SelectionOutcome<T> {
    pub fn into_inner(self) -> T {
        match self {
            SelectionOutcome::UserChose(v)
            | SelectionOutcome::TimedOutDefaulted(v)
            | SelectionOutcome::NoSelectorConfigured(v) => v,
        }
    }
}

#[async_trait::async_trait]
pub trait HostSelection: Send + Sync {
    async fn select_hls_quality(
        &self,
        task_id: &str,
        options: &[HlsQualityOption],
        timeout: Duration,
    ) -> SelectionOutcome<i32>;

    async fn select_bt_files(
        &self,
        task_id: &str,
        files: &[BtFileEntry],
        timeout: Option<Duration>,
    ) -> SelectionOutcome<Vec<i32>>;

    async fn select_resolve_variant(
        &self,
        task_id: &str,
        options: &[ResolveVariantOption],
        default_index: i32,
        timeout: Duration,
    ) -> SelectionOutcome<i32>;

    fn provide_hls_selection(&self, task_id: &str, selected_index: i32);
    fn provide_bt_selection(&self, task_id: &str, selected_indices: Vec<i32>);
    fn provide_variant_selection(&self, task_id: &str, selected_index: i32);
}
