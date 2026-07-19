mod ffmpeg;
mod ytdlp;

pub use ffmpeg::*;
pub use ytdlp::*;

pub async fn resolve_ffmpeg(data_dir: &std::path::Path) -> Option<std::path::PathBuf> {
    ffmpeg::resolve_ffmpeg(data_dir).await
}
