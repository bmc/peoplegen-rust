//! Environment variable-related helpers.

use std::env;

/**
 * Get the value of an environment variable as a `String`, returning `""`
 * if the variable isn't set (or is explicitly set to `""`).
 */
pub fn getenv(s: &str) -> String {
    // match env::var_os(s) {
    //     Some(v) => v.into_string().unwrap(),
    //     None => String::new()
    // }

    env::var_os(s)
        .map(|v| v.into_string().unwrap())
        .unwrap_or_else(|| String::new())
}
