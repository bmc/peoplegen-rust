use std::path::PathBuf;

pub fn path_str(path: &PathBuf) -> &str {
    path.as_os_str().to_str().unwrap_or("")
}
