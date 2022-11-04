/**
 * Given a string, this function parses the string into an unsigned
 * integer (`u32`). The primary reason to use this function over the
 * `String::parse()` function is that this function provides a slightly
 * better error message.
 *
 * # Arguments
 *
 * `s` - the string to convert
 *
 * # Returns
 *
 * `Ok(n)` - the parsed integral result (`n`)
 * `Err(msg)` - not a valid number, with an appropriate error message.
 */
pub fn parse_int(s: &String) -> Result<u32, String> {
    s.parse::<u32>().map_err(|_| format!("\"{s}\" is an invalid number"))
}
