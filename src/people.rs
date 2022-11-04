//!
//! Contains the definition of a generated person, plus functions to:
//!
//! - read people-related data from files
//! - randomly generate `Person` objects
//! - serialize generated data to CSV

use std::fs::File;
use std::path::PathBuf;
use std::io::{self, prelude::*};
use chrono::naive::{NaiveDate, NaiveDateTime};
use rand::Rng;
use rand::seq::SliceRandom;
use csv::WriterBuilder;
use crate::path::path_str;
use crate::args::{Arguments, HeaderFormat};

/**
 * Abstract representation of gender. Too restrictive currently, but it
 * matches the gender definitions in the 2010 Census Bureau data.
*/
#[derive(PartialEq)]
pub enum Gender {
    Male,
    Female
}

impl Gender {
    /**
        Converts a `Gender` value to a string suitable for display or for
        writing to a CSV file.
    */
    pub fn to_str(&self) -> &str {
        if *self == Gender::Male {
            "M"
         }
         else {
            "F"
         }
    }
}

/**
 * Represents a generated person.
 *
 * # Fields
 *
 * - `first_name`: The person's first name (gender-specific)
 * - `middle_name`: The person's middle name (gender-specific)
 * - `last_name`: The person's last name
 * - `gender`: The gender
 * - `birth_date`: The person's birth date
 * - `ssn`: The person's (fake) U.S. Social Security Number
*/
pub struct Person {
    pub first_name: String,
    pub middle_name: String,
    pub last_name: String,
    pub gender: Gender,
    pub birth_date: NaiveDate,
    pub ssn: String
}

/**
  * Read a file of names into a vector of strings.
  *
  * # Arguments
  *
  * - path: The path to the file to be read
  *
  * # Returns
  *
  * - `Ok(v)`: The file was successfully read into vector `v`
  * - `Err(msg)`: The file could not be read, and `msg` explains why
*/
pub fn read_names_file(path: &PathBuf) -> Result<Vec<String>, String> {
    let file = File::open(path)
        .map_err(|e| format!("\"{}\": {}", path_str(path), e))?;
    let reader = io::BufReader::new(file);
    let mut buf: Vec<String> = Vec::new();

    for line_res in reader.lines() {
        let line = line_res.map_err(|e| format!("{}", e))?;
        buf.push(line);
    }

    Ok(buf)
}

/**
 * Generate the fake people, based on the command-line settings. Note that
 * fake Social Security numbers are always generated, regardless of the
 * setting of `args.generate_ssns`. They should be suppressed at write-time,
 * if desired.
 *
 * # Arguments
 *
 * - `args`: The parsed command-line arguments. The number of people generated
 * is taken from `args.total`.
 * - `male_first_names`: The list of male first names
 * - `female_first_names`: The list of female first names
 * - `last_names`: The list of last names
 *
 * # Returns
 *
 * A vector containing the randomly generated `Person` objects.
 */
pub fn make_people(args: &Arguments,
                   male_first_names: &Vec<String>,
                   female_first_names: &Vec<String>,
                   last_names: &Vec<String>) -> Vec<Person> {
    let epoch_start = NaiveDate::from_ymd(args.year_min as i32, 1, 1)
                                .and_hms(0, 0, 0)
                                .timestamp();
    let epoch_end = NaiveDate::from_ymd(args.year_max as i32, 12, 31)
                              .and_hms(23, 59, 59)
                              .timestamp();
    let total_males: u32 = (args.total * args.male_percent) / 100;
    let w = (args.total * args.female_percent) / 100;
    let total_females = w + (args.total - total_males - w);
    let mut rng = rand::thread_rng();
    let mut ssn_prefixes: Vec<u32> = (900..=999).collect();
    ssn_prefixes.push(666);

    let mut buf: Vec<Person> = Vec::new();

    for _ in 0..total_males {
        let p = make_person(male_first_names,
                            last_names,
                            Gender::Male,
                            epoch_start,
                            epoch_end,
                            &ssn_prefixes);
        buf.push(p)
    }

    for _ in 0..total_females {
        let p = make_person(female_first_names,
                            last_names,
                            Gender::Female,
                            epoch_start,
                            epoch_end,
                            &ssn_prefixes);
        buf.push(p)
    }

    buf.shuffle(&mut rng);
    buf
}


/**
 * Creates a CSV file from a vector of randomly generated `Person` objects.
 *
 * # Arguments
 *
 * - `path`: The path to the CSV file to create or overwrite
 * - `header_format`: What style of CSV header names to use
 * - `generate_ids`: Whether or not to generate and save unique numeric IDs
 *                   for each person
 * - `save_ssns`: Whether or not to save the fake Social Security numbers
 * - `people`: The list of randomly generated people to save
 *
 * # Returns
 *
 * - `Ok(total)`: The save was successful, and `total` people were written
 * - `Err(msg)`: Unable to write the CSV file; `msg` explains why.
 */
pub fn write_people(path: &PathBuf,
                    header_format: HeaderFormat,
                    generate_ids: bool,
                    save_ssns: bool,
                    people: &Vec<Person>) -> Result<usize, String> {
    let mut w = WriterBuilder::new()
        .from_path(path)
        .map_err(|e| format!("Can't write to \"{}\": {}", path_str(path), e))?;

    let (id_header, base_headers, ssn_header) = match header_format {
        HeaderFormat::SnakeCase => {
            ("id",
             ["first_name", "middle_name", "last_name", "gender", "birth_date"],
             "ssn")
        },
        HeaderFormat::CamelCase => {
           ("id",
            ["firstName", "middleName", "lastName", "gender", "birthDate"],
            "ssn")
        },
        HeaderFormat::Pretty => {
            ("ID",
             ["First Name", "Middle Name", "Last Name", "Gender", "Birth Date"],
             "SSN")
        }
    };

    let mut header: Vec<&str> = Vec::new();

    if generate_ids {
        header.push(id_header);
    }

    header.extend(base_headers);

    if save_ssns {
        header.push(ssn_header);
    }

    w.write_record(&header).map_err(|e| format!("{}", e))?;

    for (i, p) in people.iter().enumerate() {
        let id = i + 1;
        let id_str = id.to_string();
        let mut rec: Vec<&String> = Vec::new();

        if generate_ids {
            rec.push(&id_str);
        }

        let birth_str = p.birth_date.format("%Y-%m-%d").to_string();
        let gender_str = p.gender.to_str().to_string();

        rec.extend([
            &p.first_name, &p.middle_name, &p.last_name,
            &gender_str, &birth_str
        ]);

        if save_ssns {
            rec.push(&p.ssn);
        }

        w.write_record(&rec).map_err(|e| format!("{}", e))?;
    }

    Ok(people.len())
}



/**
 * Generate a fake Social Security number. These numbers are guaranteed to
 * be unused.
 *
 * # Arguments
 *
 * - `ssn_prefixes`: The set of known unused Social Security prefixes (the
 *                   first three numbers of an SSN).
 *
 * # Returns
 *
 * The generated Social Security number.
 */
fn make_ssn(ssn_prefixes: &Vec<u32>) -> String {
    let mut rng = rand::thread_rng();
    let first_index = rng.gen_range(0..ssn_prefixes.len());
    let first = ssn_prefixes[first_index];
    let second = rng.gen_range(10..=99);
    let third = rng.gen_range(1000..=9999);

    format!("{}-{}-{}", first, second, third)
}

/**
 * Randomly generate a single `Person`.
 *
 * # Arguments
 *
 * - `first_names`: The first names from which to choose a random first name
 * - `last_names`: The last names from which to choose a random last name
 * - `gender`: The assigned gender
 * - `epoch_start`: The starting year for birth dates, as a Unix timestamp
 * - `epoch_end`: The ending year for birth dates, as a Unix timestamp
 * - `ssn_prefixes`: The set of known unused Social Security prefixes (the
 *                   first three numbers of an SSN).
 *
 * # Returns
 *
 * The generated `Person`.
 */
fn make_person(first_names: &Vec<String>,
               last_names: &Vec<String>,
               gender: Gender,
               epoch_start: i64,
               epoch_end: i64,
               ssn_prefixes: &Vec<u32>) -> Person {

    let first_index = rand::thread_rng().gen_range(0..first_names.len());
    let mid_index = rand::thread_rng().gen_range(0..first_names.len());
    let last_index = rand::thread_rng().gen_range(0..last_names.len());
    let epoch_birth = rand::thread_rng().gen_range(epoch_start..=epoch_end);
    let birth_date = NaiveDateTime::from_timestamp(epoch_birth, 0).date();

    Person {
        first_name: String::from(&first_names[first_index]),
        middle_name: String::from(&first_names[mid_index]),
        last_name: String::from(&last_names[last_index]),
        gender: gender,
        birth_date: birth_date,
        ssn: make_ssn(ssn_prefixes)
    }
}
