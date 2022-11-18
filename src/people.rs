//!
//! Contains the definition of a generated person, plus functions to:
//!
//! - read people-related data from files
//! - randomly generate `Person` objects
//! - serialize generated data to CSV

use crate::args::{Arguments, HeaderFormat, OutputFormat};
use crate::path::path_str;
use crate::ssn::SsnGenerator;
use chrono::naive::{NaiveDate, NaiveDateTime};
use csv::WriterBuilder;
use json::JsonValue;
use rand::seq::SliceRandom;
use rand::Rng;
use rand_distr::{Normal, Distribution};
use std::collections::HashMap;
use std::fs::File;
use std::io::LineWriter;
use std::io::{self, prelude::*};
use std::path::PathBuf;
use thousands::Separable;

/**
 * Abstract representation of gender. Too restrictive currently, but it
 * matches the gender definitions in the 2010 Census Bureau data.
*/
#[derive(PartialEq)]
pub enum Gender {
    Male,
    Female,
}

impl Gender {
    /**
     * Converts a `Gender` value to a string suitable for display or for
     * writing to a CSV file.
     */
    pub fn to_str(&self) -> &str {
        if *self == Gender::Male {
            "M"
        } else {
            "F"
        }
    }

    /**
     * Converts a `Gender` value to a string suitable for display or for
     * writing to a CSV file.
     */
    pub fn to_string(&self) -> String {
        String::from(self.to_str())
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
    pub ssn: String,
    pub salary: u32
}

const HEADER_ID_KEY: &str = "id";
const HEADER_FIRST_NAME_KEY: &str = "first_name";
const HEADER_LAST_NAME_KEY: &str = "last_name";
const HEADER_MIDDLE_NAME_KEY: &str = "middle_name";
const HEADER_GENDER_KEY: &str = "gender";
const HEADER_BIRTH_DATE_KEY: &str = "birth_date";
const HEADER_SSN_KEY: &str = "ssn";
const HEADER_SALARY_KEY: &str = "salary";

const REQUIRED_HEADERS: [&str; 5] = [
    HEADER_FIRST_NAME_KEY,
    HEADER_MIDDLE_NAME_KEY,
    HEADER_LAST_NAME_KEY,
    HEADER_GENDER_KEY,
    HEADER_BIRTH_DATE_KEY,
];

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
    let file = File::open(path).map_err(|e| format!("\"{}\": {}", path_str(path), e))?;
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
pub fn make_people(
    args: &Arguments,
    male_first_names: &Vec<String>,
    female_first_names: &Vec<String>,
    last_names: &Vec<String>,
) -> Result<Vec<Person>, String> {
    let epoch_start = NaiveDate::from_ymd(args.year_min as i32, 1, 1)
        .and_hms(0, 0, 0)
        .timestamp();
    let epoch_end = NaiveDate::from_ymd(args.year_max as i32, 12, 31)
        .and_hms(23, 59, 59)
        .timestamp();
    let male_percent = args.male_percent as u64;
    let female_percent = args.female_percent as u64;
    let total_males: u64 = (args.total * male_percent) / 100;
    let w = (args.total * female_percent) / 100;
    let total_females = w + (args.total - total_males - w);
    let mut rng = rand::thread_rng();
    let mut ssns = SsnGenerator::new_auto_reset();

    if args.total > ssns.total() {
        println!(
"Warning: There are {} total unique SSNs.
You're generating {} people.
There will be some repeated SSNs.",
ssns.total().separate_with_commas(),
args.total.separate_with_commas())
    }

    let normal_dist =
        Normal::new(args.salary_mean as f32, args.salary_sigma as f32)
              .map_err(|e| format!("{}", e))?;
    let mut get_salary = || {
        let s = normal_dist.sample(&mut rng);
        if s < 0.0 {
            Err(format!("Generated negative salary ({s})"))
        }
        else {
            Ok(s as u32)
        }
    };

    let mut buf: Vec<Person> = Vec::new();

    for _ in 0..total_males {
        let salary = get_salary()?;
        let ssn = ssns.next().unwrap();
        let p = make_person(
            male_first_names,
            last_names,
            Gender::Male,
            salary,
            epoch_start,
            epoch_end,
            ssn,
        );
        buf.push(p)
    }

    for _ in 0..total_females {
        let salary = get_salary()?;
        let ssn = ssns.next().unwrap();
        let p = make_person(
            female_first_names,
            last_names,
            Gender::Female,
            salary,
            epoch_start,
            epoch_end,
            ssn,
        );
        buf.push(p)
    }

    buf.shuffle(&mut rng);
    Ok(buf)
}

/**
 * Creates a CSV or JSON file from a vector of randomly generated `Person`
 * objects.
 *
 * # Arguments
 *
 * - `path`: The path to the CSV file to create or overwrite
 * - `output_format`: The output format of the file
 * - `header_format`: What style of CSV header names or JSON keys to use.
 * - `generate_ids`: Whether or not to generate and save unique numeric IDs
 *                   for each person
 * - `save_ssns`: Whether or not to save the fake Social Security numbers
 * - `save_salaries`: Whether or not to save generated salary data
 * - `people`: The list of randomly generated people to save. Note that this
 *             parameter isn't a reference and is, therefore, consumed by this
 *             function.
 *
 * # Returns
 *
 * - `Ok(total)`: The save was successful, and `total` people were written
 * - `Err(msg)`: Unable to write the CSV file; `msg` explains why.
 */
pub fn write_people(
    path: &PathBuf,
    output_format: OutputFormat,
    header_format: HeaderFormat,
    generate_ids: bool,
    save_ssns: bool,
    save_salaries: bool,
    people: Vec<Person>,
) -> Result<usize, String> {
    match output_format {
        OutputFormat::Csv => {
            write_csv(path, header_format, generate_ids, save_ssns,
                      save_salaries, people)
        },
        OutputFormat::JsonL => {
            write_jsonl(path, header_format, generate_ids, save_ssns,
                        save_salaries, people)
        },
        OutputFormat::JsonPretty => {
            write_json(path, header_format, generate_ids, save_ssns,
                       save_salaries, people)
        }
    }
}

// ----------------------------------------------------------------------------
// Private Members
// ----------------------------------------------------------------------------

/**
 * Creates a JSON Lines file from a vector of randomly generated `Person`
 * objects. JSON Lines is a line-by-line JSON format, where each object
 * occupies its own text line, and there's no enclosing object or array. For
 * instance:
 *
 * ```
 * { "first_name": "Moe", ... },
 * { "first_name": "Larry", ... },
 * { "first_name": "Curly", ... },
 * ...
 * ```
 *
 * JSON files of this form are well-suited for ingesting into distributed
 * systems such as Apache Spark, for processing with line-based Unix tools,
 * etc.
 *
 * # Arguments
 *
 * - `path`: The path to the JSON file to create or overwrite
 * - `header_format`: What style of JSON keys to use.
 * - `generate_ids`: Whether or not to generate and save unique numeric IDs
 *                   for each person
 * - `save_ssns`: Whether or not to save the fake Social Security numbers
 * - `save_salaries`: Whether or not to save generated salary data
 * - `people`: The list of randomly generated people to save. Note that this
 *             parameter isn't a reference and is, therefore, consumed by this
 *             function.
 *
 * # Returns
 *
 * - `Ok(total)`: The save was successful, and `total` people were written
 * - `Err(msg)`: Unable to write the CSV file; `msg` explains why.
 */

fn write_jsonl(
    path: &PathBuf,
    header_format: HeaderFormat,
    generate_ids: bool,
    save_ssns: bool,
    save_salaries: bool,
    people: Vec<Person>,
) -> Result<usize, String> {
    let file = File::create(path).map_err(|e| format!("{}", e))?;
    let mut w = LineWriter::new(file);
    let headers = get_headers(header_format);

    for (i, p) in people.iter().enumerate() {
        let id = if generate_ids { Some(i + 1) } else {None};

        let jv = person_to_json_object(p, &headers, id, save_ssns, save_salaries)?;

        let json_line = jv.dump();

        w.write_fmt(format_args!("{}\n", json_line))
            .map_err(|e| format!("Can't write to \"{}\": {}", path_str(path), e))?;
    }

    Ok(people.len())
}

/**
 * Creates a JSON document from a vector of randomly generated `Person` objects.
 * The JSON output is of this form (though _not_ pretty-printed):
 *
 * ```
 * {"people": [
 *   { "first_name": "Moe", ... },
 *   { "first_name": "Larry", ... },
 *   { "first_name": "Curly", ... },
 *   ...
 * ]}
 * ```
 *
 * # Arguments
 *
 * - `path`: The path to the JSON file to create or overwrite
 * - `header_format`: What style of JSON keys to use.
 * - `generate_ids`: Whether or not to generate and save unique numeric IDs
 *                   for each person
 * - `save_ssns`: Whether or not to save the fake Social Security numbers
 * - `save_salaries`: Whether or not to save generated salary data
 * - `people`: The list of randomly generated people to save. Note that this
 *             parameter isn't a reference and is, therefore, consumed by this
 *             function.
 *
 * # Returns
 *
 * - `Ok(total)`: The save was successful, and `total` people were written
 * - `Err(msg)`: Unable to write the CSV file; `msg` explains why.
 */
fn write_json(
    path: &PathBuf,
    header_format: HeaderFormat,
    generate_ids: bool,
    save_ssns: bool,
    save_salaries: bool,
    people: Vec<Person>,
) -> Result<usize, String> {

    let file = File::create(path).map_err(|e| format!("{}", e))?;
    let mut w = LineWriter::new(file);
    let headers = get_headers(header_format);
    let mut jo = JsonValue::new_object();
    let mut ja = JsonValue::new_array();

    for (i, p) in people.iter().enumerate() {
        let id = if generate_ids { Some(i + 1) } else {None};

        let jv = person_to_json_object(p, &headers, id, save_ssns, save_salaries)?;
        ja.push(jv).map_err(|e| format!("{}", e))?;
    }

    jo.insert("people", ja).map_err(|e| format!("{}", e))?;

    let json_str = jo.dump();
    w.write_fmt(format_args!("{}\n", json_str))
        .map_err(|e| format!("Can't write to \"{}\": {}", path_str(path), e))?;

    Ok(people.len())
}

/**
 * Creates a CSV from a vector of randomly generated `Person` objects.
 *
 * # Arguments
 *
 * - `path`: The path to the CSV file to create or overwrite
 * - `header_format`: What style of CSV header names or JSON keys to use.
 * - `generate_ids`: Whether or not to generate and save unique numeric IDs
 *                   for each person
 * - `save_ssns`: Whether or not to save the fake Social Security numbers
 * - `people`: The list of randomly generated people to save. Note that this
 *             parameter isn't a reference and is, therefore, consumed by this
 *             function.
 *
 * # Returns
 *
 * - `Ok(total)`: The save was successful, and `total` people were written
 * - `Err(msg)`: Unable to write the CSV file; `msg` explains why.
 */
fn write_csv(
    path: &PathBuf,
    header_format: HeaderFormat,
    generate_ids: bool,
    save_ssns: bool,
    save_salaries: bool,
    people: Vec<Person>,
) -> Result<usize, String> {

    let mut w = WriterBuilder::new()
        .from_path(path)
        .map_err(|e| format!("Can't write to \"{}\": {}", path_str(path), e))?;

    let headers = get_headers(header_format);

    let mut header_rec: Vec<&String> = Vec::new();

    if generate_ids {
        header_rec.push(headers.get(HEADER_ID_KEY).unwrap())
    }

    for h in REQUIRED_HEADERS {
        header_rec.push(headers.get(h).unwrap())
    }

    if save_ssns {
        header_rec.push(headers.get(HEADER_SSN_KEY).unwrap());
    }

    if save_salaries {
        header_rec.push(headers.get(HEADER_SALARY_KEY).unwrap());
    }

    w.write_record(&header_rec).map_err(|e| format!("{}", e))?;

    for (i, p) in people.iter().enumerate() {
        let id = i + 1;
        let id_str = id.to_string();
        let mut rec: Vec<&String> = Vec::new();
        let salary = p.salary.to_string();

        if generate_ids {
            rec.push(&id_str);
        }

        let birth_str = date_str(&p.birth_date);
        let gender_str = p.gender.to_str().to_string();

        rec.extend([
            &p.first_name,
            &p.middle_name,
            &p.last_name,
            &gender_str,
            &birth_str,
        ]);

        if save_ssns {
            rec.push(&p.ssn);
        }

        if save_salaries {
            rec.push(&salary);
        }

        w.write_record(&rec).map_err(|e| format!("{}", e))?;
    }

    Ok(people.len())
}

/**
 * Map a `Person` object to a JSON `JsonValue`.
 *
 * # Arguments
 *
 * - `person`: The `Person` object
 * - `headers`: A map of the keys to use, from `get_headers()`
 * - `opt_id`: A `Some` with the generated ID for the user, or `None` for no ID
 * - `save_ssn`: Whether or not to save the Social Security number
 *
 * # Returns
 *
 * - `Ok(JsonValue)` if the conversion worked
 * - `Err(msg)` if it failed
 */
fn person_to_json_object(
    person: &Person,
    headers: &HashMap<&str, String>,
    opt_id: Option<usize>,
    save_ssn: bool,
    save_salary: bool
) -> Result<JsonValue, String> {
    let id_key = headers.get(HEADER_ID_KEY).unwrap();
    let first_name_key = headers.get(HEADER_FIRST_NAME_KEY).unwrap();
    let middle_name_key = headers.get(HEADER_MIDDLE_NAME_KEY).unwrap();
    let last_name_key = headers.get(HEADER_LAST_NAME_KEY).unwrap();
    let gender_key = headers.get(HEADER_GENDER_KEY).unwrap();
    let birth_date_key = headers.get(HEADER_BIRTH_DATE_KEY).unwrap();
    let ssn_key = headers.get(HEADER_SSN_KEY).unwrap();
    let salary_key = headers.get(HEADER_SALARY_KEY).unwrap();

    let mut rec = JsonValue::new_object();
    let s_id = opt_id.map(|i| i.to_string());
    let s_gender = person.gender.to_string();
    let s_date = date_str(&person.birth_date);

    // Have to clone each of the people fields, because the JsonValue
    // object wants to capture them (and doesn't support &String).
    let first_name = person.first_name.to_string();
    let middle_name = person.middle_name.to_string();
    let last_name = person.last_name.to_string();
    let ssn = person.ssn.to_string();
    let salary = person.salary.to_string();

    if let Some(s) = s_id {
        rec.insert(&id_key, s).map_err(|e| format!("{}", e))?;
    }

    rec.insert(&first_name_key, first_name)
        .map_err(|e| format!("{}", e))?;
    rec.insert(&middle_name_key, middle_name)
        .map_err(|e| format!("{}", e))?;
    rec.insert(&last_name_key, last_name)
        .map_err(|e| format!("{}", e))?;
    rec.insert(&gender_key, s_gender)
        .map_err(|e| format!("{}", e))?;
    rec.insert(&birth_date_key, s_date)
        .map_err(|e| format!("{}", e))?;

    if save_ssn {
        rec.insert(&ssn_key, ssn).map_err(|e| format!("{}", e))?;
    }

    if save_salary {
        rec.insert(&salary_key, salary).map_err(|e| format!("{}", e))?;
    }

    Ok(rec)
}

fn date_str(d: &NaiveDate) -> String {
    d.format("%Y-%m-%d").to_string()
}

fn get_headers(header_format: HeaderFormat) -> HashMap<&'static str, String> {
    let mut m: HashMap<&str, String> = HashMap::new();

    match header_format {
        HeaderFormat::SnakeCase => {
            m.insert(HEADER_ID_KEY, String::from("id"));
            m.insert(HEADER_FIRST_NAME_KEY, String::from("first_name"));
            m.insert(HEADER_MIDDLE_NAME_KEY, String::from("middle_name"));
            m.insert(HEADER_LAST_NAME_KEY, String::from("last_name"));
            m.insert(HEADER_GENDER_KEY, String::from("gender"));
            m.insert(HEADER_BIRTH_DATE_KEY, String::from("birth_date"));
            m.insert(HEADER_SSN_KEY, String::from("ssn"));
            m.insert(HEADER_SALARY_KEY, String::from("salary"));
        }
        HeaderFormat::CamelCase => {
            m.insert(HEADER_ID_KEY, String::from("id"));
            m.insert(HEADER_FIRST_NAME_KEY, String::from("firstName"));
            m.insert(HEADER_MIDDLE_NAME_KEY, String::from("middleName"));
            m.insert(HEADER_LAST_NAME_KEY, String::from("lastName"));
            m.insert(HEADER_GENDER_KEY, String::from("gender"));
            m.insert(HEADER_BIRTH_DATE_KEY, String::from("birthDate"));
            m.insert(HEADER_SSN_KEY, String::from("ssn"));
            m.insert(HEADER_SALARY_KEY, String::from("salary"));
        }
        HeaderFormat::Pretty => {
            m.insert(HEADER_ID_KEY, String::from("ID"));
            m.insert(HEADER_FIRST_NAME_KEY, String::from("First Name"));
            m.insert(HEADER_MIDDLE_NAME_KEY, String::from("Middle Name"));
            m.insert(HEADER_LAST_NAME_KEY, String::from("Last Name"));
            m.insert(HEADER_GENDER_KEY, String::from("Gender"));
            m.insert(HEADER_BIRTH_DATE_KEY, String::from("Birth Date"));
            m.insert(HEADER_SSN_KEY, String::from("SSN"));
            m.insert(HEADER_SALARY_KEY, String::from("Salary"));
        }
    };

    m
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
fn make_person(
    first_names: &Vec<String>,
    last_names: &Vec<String>,
    gender: Gender,
    salary: u32,
    epoch_start: i64,
    epoch_end: i64,
    ssn: String
) -> Person {
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
        ssn,
        salary
    }
}
