//! Some simple numeric helpers.

use funty::Integral;

/// Given a string, this function parses the string into an integer. The
/// function is generic and should support all primitive integral types.
/// The primary reason to use this function over the `String::parse()` or
/// something like `u32::from_str_radix()" is that, on error, this function
/// returns an already-formatted informative error message, one that's more
/// useful than the one from `String::parse()`. (the `::from_str_radix()`
/// functions don't return a string at all; they return an error struct.)
///
/// # Arguments
///
/// `s` - the string to convert
/// `radix` - the radix or base
///
/// # Returns
///
/// `Ok(n)` - the parsed integral result (`n`)
/// `Err(msg)` - not a valid number, with an appropriate error message.
pub fn parse_int<T>(s: &String, radix: u32) -> Result<T, String>
    where T: Integral
{
    T::from_str_radix(s, radix)
        .map_err(|_| format!(
            "\"{}\" is not a valid base-{} number for this type",
             s, radix
            )
        )

    //s.parse::<T>().map_err(|_| format!("\"{s}\" is an invalid number"))
}

#[cfg(test)]
mod tests {
    use crate::numlib::parse_int;

    #[test]
    fn parse_u32() {
        assert_eq!(parse_int::<u32>(&String::from("0"), 10), Ok(0));
        assert!(parse_int::<u32>(&String::from("-1"), 10).is_err());
        assert_eq!(parse_int::<u32>(&String::from("948135734"), 10), Ok(948135734));
        assert!(parse_int::<u32>(&String::from("foobar"), 10).is_err());
    }

    #[test]
    fn parse_u8() {
        assert_eq!(parse_int::<u8>(&String::from("0"), 10), Ok(0));
        assert!(parse_int::<u8>(&String::from("-1"), 10).is_err());
        assert!(parse_int::<u8>(&String::from("948135734"), 10).is_err());
        assert!(parse_int::<u8>(&String::from("foobar"), 10).is_err());
    }

    #[test]
    fn parse_i32() {
        assert_eq!(parse_int::<i32>(&String::from("0"), 10), Ok(0));
        assert_eq!(parse_int::<i32>(&String::from("-1"), 10), Ok(-1));
        assert_eq!(parse_int::<i32>(&String::from("948135734"), 10), Ok(948135734));
        assert!(parse_int::<i32>(&String::from("foobar"), 10).is_err());
    }

    #[test]
    fn parse_hex_u32() {
        assert_eq!(parse_int::<u32>(&String::from("ffff"), 16), Ok(65535));
        assert_eq!(parse_int::<u32>(&String::from("00a0"), 16), Ok(160));
        assert!(parse_int::<u32>(&String::from("x00a0"), 16).is_err());
    }
}
