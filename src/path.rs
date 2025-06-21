//! Some path-related helpers.

use std::path::PathBuf;
use std::ffi::OsStr;

/// Simple utility function to take a reference to a `PathBuf` and return
/// the pathname as a normal string (i.e., not an `OsString` or an `OsStr`),
/// usually for printing.
///
/// # Arguments:
///
/// - `path`: The `PathBuf`
///
/// # Returns
///
/// The pathname as a string, or "" if the pathname could not be decoded (which,
/// frankly, should rarely, if ever, happen)
pub fn path_str(path: &PathBuf) -> &str {
    path.as_os_str().to_str().unwrap_or("")
}

/// Convenience function that determines whether a path is empty.
///
/// # Arguments
///
/// - `p`: The path to check
///
/// # Returns
///
/// - `true` if the path is empty (i.e., `PathBuf::as_os_str()` returns an empty
///   `OsStr`)
/// - `false` if the path is not empty
pub fn path_is_empty(p: &PathBuf) -> bool {
    p.as_path().as_os_str() == ""
}

/// Convenience function to get a file extension. Unlike `PathBuf::extension()`,
/// this function returns a `&str` (rather than an `OsStr`).
///
/// # Arguments
///
/// - `path`: The path to query
///
/// # Returns
///
/// - `Some(extension)` if there's a file extension
/// - `None` if there's no file extension
pub fn file_extension(path: &PathBuf) -> Option<&str> {
    path.extension().and_then(OsStr::to_str)
}

