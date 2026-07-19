use std::path::Path;

pub async fn resolve_ffmpeg(data_dir: &Path) -> Option<std::path::PathBuf> {
    // Try: bundled in data_dir, then system PATH
    let bundled = data_dir.join("ffmpeg");
    #[cfg(windows)]
    let bundled = bundled.with_extension("exe");
    if bundled.exists() { return Some(bundled); }

    // Fallback: check PATH
    which_ffmpeg().await
}

async fn which_ffmpeg() -> Option<std::path::PathBuf> {
    tokio::task::spawn_blocking(|| {
        std::process::Command::new("ffmpeg").arg("-version").output().ok()
    }).await.ok().and_then(|output| {
        let output = output?;
        if output.status.success() {
            // Find the path
            std::env::var_os("PATH")
                .and_then(|paths| {
                    std::env::split_paths(&paths).find_map(|dir| {
                        let candidate = dir.join("ffmpeg");
                        if candidate.exists() { Some(candidate) } else { None }
                    })
                })
        } else { None }
    })
}

pub async fn probe_ffmpeg(data_dir: &Path) -> bool {
    resolve_ffmpeg(data_dir).await.is_some()
}

pub async fn install_ffmpeg(_data_dir: &Path) -> Result<(), String> {
    Err("ffmpeg auto-install not implemented in this version".to_string())
}
