use std::fs::File;
//use std::io;
use std::process;
use crate::args::{Arguments, HeaderFormat, parse_args};
use crate::people::{Person, read_first_names, read_last_names, make_person,
                    Gender};

pub mod numlib;
pub mod args;
pub mod people;
pub mod path;

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
    read_first_names(&args.first_names_file)
      .and_then(|(female_first_names, male_first_names)| {
        read_last_names(&args.last_names_file)
          .map(|last_names| (female_first_names, male_first_names, last_names))
      })
      .and_then(|t| {
        let (female_first_names, male_first_names, last_names) = t;
        println!("Read {} female first names", female_first_names.len());
        println!("Read {} male first names", male_first_names.len());
        println!("Read {} last names", last_names.len());
        Ok(make_person(male_first_names, last_names, Gender::Male))
      })
      .and_then(|p: Person| {
        println!("First name: {}", p.first_name);
        Ok(())
      })
}





