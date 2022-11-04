//! Some path-related helpers.

use std::path::PathBuf;

/**
 * Simple utility function to take a reference to a `PathBuf` and return
 * the pathname as a normal string (i.e., not an `OsString` or an `OsStr`),
 * usually for printing.
 *
 * # Arguments:
 *
 * - `path`: The `PathBuf`
 *
 * # Returns
 *
 * The pathname as a string, or "" if the pathname could not be decoded (which,
 * frankly, should rarely, if ever, happen)
*/
pub fn path_str(path: &PathBuf) -> &str {
    path.as_os_str().to_str().unwrap_or("")
}

/**
 * Convenience function that determines whether a path is empty.
 */
pub fn path_is_empty(p: &PathBuf) -> bool {
    p.as_path().as_os_str() == ""
}
