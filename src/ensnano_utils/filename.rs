use chrono::Local;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub const DEFAULT_FILE_PREFIX: &str = "export";
pub const DEFAULT_FILE_EXTENSION: &str = "none";

pub fn derive_path_with_prefix_and_time_stamp_and_suffix(
    from_path: Option<Arc<Path>>,
    prefix: Option<&str>,
    suffix: Option<&str>,
    extension: Option<&str>,
) -> PathBuf {
    let time_stamp = Local::now().format("%Y_%m_%d-%H_%M_%S-%6f").to_string();
    let prefix = prefix.unwrap_or(DEFAULT_FILE_PREFIX);
    let suffix = if let Some(suf) = suffix {
        format!("-{suf}")
    } else {
        "".to_string()
    };
    let extension = extension.unwrap_or(DEFAULT_FILE_EXTENSION);
    match from_path {
        Some(path) => {
            let file_stem = if let Some(stem) = path.file_stem() {
                stem.to_str().unwrap().to_owned() + "-"
            } else {
                "".to_string()
            };
            let file_name = format!("{file_stem}{prefix}-{time_stamp}{suffix}.{extension}");
            path.with_file_name(file_name)
        }
        None => {
            let file_name = format!("{prefix}-{time_stamp}{suffix}.{extension}");
            PathBuf::from(file_name)
        }
    }
}
