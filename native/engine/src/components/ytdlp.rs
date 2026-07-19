use std::path::Path;

pub async fn resolve_ytdlp(data_dir: &Path) -> Option<std::path::PathBuf> {
    let bundled = data_dir.join("yt-dlp");
    #[cfg(windows)]
    let bundled = bundled.with_extension("exe");
    if bundled.exists() { return Some(bundled); }
    None
}

pub async fn probe_ytdlp(data_dir: &Path) -> bool {
    resolve_ytdlp(data_dir).await.is_some()
}
