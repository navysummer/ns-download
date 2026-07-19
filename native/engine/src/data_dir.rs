use std::path::{Path, PathBuf};

#[derive(Debug, thiserror::Error)]
pub enum DataDirError {
    #[error("data dir not found and cannot be created: {0}")]
    Io(#[from] std::io::Error),
}

pub fn resolve_data_dir(override_dir: Option<&Path>) -> Result<PathBuf, DataDirError> {
    if let Some(dir) = override_dir {
        std::fs::create_dir_all(dir)?;
        return Ok(dir.to_path_buf());
    }
    if let Some(data_dir) = dirs::data_dir() {
        let dir = data_dir.join("ns-download");
        std::fs::create_dir_all(&dir)?;
        return Ok(dir);
    }
    let fallback = PathBuf::from("./ns-download-data");
    std::fs::create_dir_all(&fallback)?;
    Ok(fallback)
}
