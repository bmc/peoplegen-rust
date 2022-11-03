//use std::io;
use std::process;
use crate::args::{Arguments, HeaderFormat, parse_args};

pub mod numlib;
pub mod path;
pub mod args;

fn main() {
    match parse_args()
        .and_then(|args| run(args)) {
        Ok(_) => process::exit(0),
        Err(msg) => {
            eprintln!{"{}", msg};
            process::exit(1);
        }
    }
}

fn run(args: Arguments) -> Result<(), String> {
    println!("{:?}", args);
    Err(String::from("N/I"))
}
