//! Main program (and the crate root).
//!
use std::process;
use crate::args::{Arguments, parse_args};
use crate::people::{read_names_file, make_people, write_people};

#[macro_use]
extern crate comp;

pub mod numlib;
pub mod args;
pub mod people;
pub mod path;
pub mod env;

/**
 * Main program.
 */
fn main() {
    let res = result! {
        let args <- parse_args();
        run(args)
    };

    match res {
        Ok(_) => process::exit(0),
        Err(msg) => {
            eprintln!{"{}", msg};
            process::exit(1);
        }
    }
}

/**
 * `run` implements the main logic of the program, once command-line arguments
 * have been parsed.
 *
 * # Arguments
 *
 * - `args`: The parsed command-line arguments
 *
 * # Returns
 *
 * - `Ok(())`: Everything worked. No result.
 * - `Err(msg)`: Something failed, and `msg` explains the error.
 */
fn run(args: Arguments) -> Result<(), String> {
    result! {
        // The macro requires <- for "assignments" that return Result.
        let male_first_names <- read_names_file(&args.male_first_names_file);
        let female_first_names <- read_names_file(&args.female_first_names_file);
        let last_names <- read_names_file(&args.last_names_file);
        let people = make_people(
            &args,
            &male_first_names,
            &female_first_names,
            &last_names
        );
        let total <- write_people(
            &args.output_file,
            args.header_format,
            args.generate_ids,
            args.generate_ssns,
            people
        );

        println!("Wrote {} records(s) to \"{}\".",
                 total, args.output_file.display());
        ()
    }
}


