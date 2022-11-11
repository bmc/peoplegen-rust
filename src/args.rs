//! All things command-line for `peoplegen`, including the argument parser.

use std::path::PathBuf;
use std::collections::HashMap;
use clap::{Command, Arg, ArgAction};
use chrono::{Duration, Utc, Datelike};
use crate::path::{path_is_empty, file_extension};
use crate::env::getenv;

const STARTING_YEAR_DEFAULT_DELTA: u32 = 90;
const ENDING_YEAR_DEFAULT_DELTA: u32 = 18;
const ENV_MALE_FIRST_NAMES_FILE: &str = "PEOPLEGEN_MALE_FIRST_NAMES";
const ENV_FEMALE_FIRST_NAMES_FILE: &str = "PEOPLEGEN_FEMALE_FIRST_NAMES";
const ENV_LAST_NAMES_FILE: &str = "PEOPLEGEN_LAST_NAMES";

// 2021 mean salary across all professions, per the Bureau of Labor Statistics.
// See https://www.bls.gov/oes/current/oes_nat.htm
//
// These next two constants are strings, mostly to keep the argument parser
// happy.
const SALARY_MEAN_DEFAULT: &str = "58260";

// This is arbitrary
const SALARY_SIGMA_DEFAULT: &str = "5000";

/// The header format to use in the output CSV file.
#[derive(Debug, Copy, Clone)]
pub enum HeaderFormat {
    SnakeCase,
    CamelCase,
    Pretty
}

// The desired output format
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum OutputFormat {
    JsonPretty,
    JsonL,
    Csv
}

impl OutputFormat {
    /**
     * Returns a `str` representation of an output format, suitable for
     * printing or formatting.
     */
    pub fn to_str(&self) -> &str {
        match self {
            OutputFormat::JsonPretty => "JSON",
            OutputFormat::JsonL => "JSON Lines",
            OutputFormat::Csv => "CSV",
        }
    }
}

/// Command-line arguments, as parsed.
#[derive(Debug)]
pub struct Arguments {
    pub female_percent: u32,
    pub male_percent: u32,
    pub generate_ssns: bool,
    pub generate_ids: bool,
    pub generate_salaries: bool,
    pub salary_mean: u32,
    pub salary_sigma: u32,
    pub header_format: HeaderFormat,
    pub year_min: u32,
    pub year_max: u32,
    pub male_first_names_file: PathBuf,
    pub female_first_names_file: PathBuf,
    pub last_names_file: PathBuf,
    pub output_file: PathBuf,
    pub output_format: OutputFormat,
    pub total: u32
}

/**
 * Parse the command line arguments into an `Arguments` structure.
 * Returns an `Ok` with the parsed arguments, or an `Err` with a message
 * on error.
*/
pub fn parse_args() -> Result<Arguments, String> {
    let header_format_map: HashMap<&str, HeaderFormat> = HashMap::from([
        ("snake", HeaderFormat::SnakeCase),
        ("pretty", HeaderFormat::Pretty),
        ("camel", HeaderFormat::CamelCase),
    ]);
    // See https://stackoverflow.com/a/56724224/53495
    let header_formats: Vec<&str> = header_format_map.keys().cloned().collect();

    // This has to be a closure to capture header_format_map.
    let parse_header_format = |s: &String| {
        header_format_map
          .get(&s.as_str())
          .map_or(Err(format!("Bad header format value: \"{s}\"")), |h| Ok(*h))
    };

    let default_year_min = year_before_now(STARTING_YEAR_DEFAULT_DELTA);
    let default_year_max = year_before_now(ENDING_YEAR_DEFAULT_DELTA);
    let female_first_names_default = getenv(ENV_FEMALE_FIRST_NAMES_FILE);
    let male_first_names_default = getenv(ENV_MALE_FIRST_NAMES_FILE);
    let last_names_default = getenv(ENV_LAST_NAMES_FILE);

    let parser = Command::new("peoplegen")
        .version(env!("CARGO_PKG_VERSION"))
        .author("bmc@clapper.org")
        .about("Generate fake people data in a CSV")
        .arg(Arg::new("female")
                 .short('f')
                 .long("female-pct")
                 .default_value("50")
                 .value_name("PERCENT")
                 .value_parser(clap::value_parser!(u32))
                 .help("Percentage of female names."))
        .arg(Arg::new("male")
                 .short('m')
                 .long("male-pct")
                 .default_value("50")
                 .value_name("PERCENT")
                 .value_parser(clap::value_parser!(u32))
                 .help("Percentage of male names."))
        .arg(Arg::new("female-first-names")
                 .short('F')
                 .long("female-names")
                 .value_name("<path>")
                 .help(format!(
"Path to text file containing female first names, one per line.
If not specified, it defaults to the value of environment variable
{}.", ENV_FEMALE_FIRST_NAMES_FILE)))
        .arg(Arg::new("male-first-names")
                 .short('M')
                 .long("male-names")
                 .value_name("<path>")
                 .help(format!(
"Path to text file containing male first names, one per line.
If not specified, it defaults to the value of environment variable
{}.", ENV_MALE_FIRST_NAMES_FILE)))
             .arg(Arg::new("last-names")
                 .short('L')
                 .long("last-names")
                 .value_name("PATH")
                 .help(format!(
"Path to text file containing last names, one per line. If not
specified, defaults to the value of environment variable
{}.", ENV_LAST_NAMES_FILE)))
        .arg(Arg::new("ssn")
                 .short('s')
                 .long("ssn")
                 .action(ArgAction::SetTrue)
                 .help("Generate fake (and invalid) Social Security numbers"))
        .arg(Arg::new("salary")
                 .short('S')
                 .long("salary")
                 .action(ArgAction::SetTrue)
                 .help(format!(
"Generate a salary for each person. Salaries are generated as
a normal (Poisson) distribution with a mean salary of {} (the
2021 mean salary for all professions, according to the Bureau of
Labor Statistics) and a sigma (standard deviation) of {}. You can
change those values with --salary-mean and --salary-sigma",
SALARY_MEAN_DEFAULT, SALARY_SIGMA_DEFAULT)))
        .arg(Arg::new("salary-mean")
                 .long("salary-mean")
                 .value_parser(clap::value_parser!(u32))
                 .default_value(SALARY_MEAN_DEFAULT)
                 .help(format!("Mean salary to use.")))
        .arg(Arg::new("salary-sigma")
                 .long("salary-sigma")
                 .value_parser(clap::value_parser!(u32))
                 .default_value(SALARY_SIGMA_DEFAULT)
                 .help(format!("Sigma (standard deviation) for salaries.")))
        .arg(Arg::new("id")
                 .short('i')
                 .long("id")
                 .action(ArgAction::SetTrue)
                 .help("Generate unique IDs for each person"))
        .arg(Arg::new("header-format")
                 .short('H')
                 .long("header-format")
                 .default_value("snake")
                 .help(format!("CSV header format, one of: {}",
                               header_formats.join(", "))))
        .arg(Arg::new("year-min")
                 .short('y')
                 .long("year-min")
                 .value_parser(clap::value_parser!(u32))
                 .help(format!("The starting year for birth dates. Default: {}",
                       default_year_min)))
        .arg(Arg::new("year-max")
                 .short('Y')
                 .long("year-max")
                 .value_parser(clap::value_parser!(u32))
                 .help(format!("The ending year for birth dates. Default: {}",
                       default_year_max)))
        .arg(Arg::new("output")
                 .required(true)
                 .value_name("OUTPUT_FILE")
                 .help("Path to output file"))
        .arg(Arg::new("total")
                 .required(true)
                 .value_name("TOTAL")
                 .value_parser(clap::value_parser!(u32))
                 .help("How many people to generate"))
        .after_help(
"Supports CSV, JSON, and JSON Lines output formats. The output format is
determined by the output file extension (\".csv\", \".json\", or \".jsonl\").
See https://github.com/bmc/peoplegen-rust for more information.");

    let matches = parser.get_matches();

    // NOTE: It's okay to use unwrap() rather than unwrap_or() on arguments
    // with a default, because they'll never come back as None.
    let female_percent = matches
        // In this case, get_one() actually returns a reference to a u32.
        // We can use map() to dereference it.
        .get_one::<u32>("female")
        .map(|reference| *reference)
        .unwrap();
    let male_percent = matches
        .get_one::<u32>("male")
        .map(|reference| *reference)
        .unwrap();
    let year_min = matches
        .get_one::<u32>("year-min")
        .map(|reference| *reference)
        .unwrap_or_else(|| year_before_now(STARTING_YEAR_DEFAULT_DELTA));
    let year_max = matches
        .get_one::<u32>("year-max")
        .map(|reference| *reference)
        .unwrap_or_else(|| year_before_now(ENDING_YEAR_DEFAULT_DELTA));
    let header_format = matches
        .get_one::<String>("header-format")
        .map(|s| parse_header_format(s))
        .unwrap()?;
    let male_first_names_file = matches
        .get_one::<String>("male-first-names")
        .unwrap_or(&male_first_names_default);
    let female_first_names_file = matches
        .get_one::<String>("female-first-names")
        .unwrap_or(&female_first_names_default);
    let last_names_file = matches
        .get_one::<String>("last-names")
        .unwrap_or(&last_names_default);
    let salary_mean = matches
        .get_one::<u32>("salary-mean")
        .map(|reference| *reference)
        .unwrap();
    let salary_sigma = matches
        .get_one::<u32>("salary-sigma")
        .map(|reference| *reference)
        .unwrap();
    let output_file = matches
        .get_one::<String>("output")
        .map(PathBuf::from)
        .unwrap();
    let total = matches
        .get_one::<u32>("total")
        .map(|reference| *reference)
        .unwrap();

    let output_format = match file_extension(&output_file) {
        Some("csv") => Ok(OutputFormat::Csv),
        Some("json") => Ok(OutputFormat::JsonPretty),
        Some("jsonl") => Ok(OutputFormat::JsonL),
        Some(_) | None => Err(format!(
            "Output file \"{}\" must end in \".csv\", \".json\" or \".jsonl\".",
            output_file.display()
        )),
    }?;

    validate(Arguments {
        female_percent,
        male_percent,
        generate_ssns: *matches.get_one::<bool>("ssn").unwrap(),
        generate_ids: *matches.get_one::<bool>("id").unwrap(),
        generate_salaries: *matches.get_one::<bool>("salary").unwrap(),
        salary_mean,
        salary_sigma,
        header_format,
        year_min,
        year_max,
        male_first_names_file: PathBuf::from(male_first_names_file),
        female_first_names_file: PathBuf::from(female_first_names_file),
        last_names_file: PathBuf::from(last_names_file),
        output_format,
        output_file: output_file,
        total
    })
}

/// Given the current date, return the year `years` ago.
fn year_before_now(years: u32) -> u32 {
    // There's no Duration::years(), so just use weeks and multiply.
    let y = years as i64;
    (Utc::now() - Duration::weeks(y * 52)).year() as u32
}

/// Cross-validate the parsed arguments.
fn validate(args: Arguments) -> Result<Arguments, String> {
    if (args.female_percent + args.male_percent) != 100 {
        Err(String::from("Female and male percentages must add up to 100."))
    }

    else if args.year_min > args.year_max {
        Err(String::from(format!(
            "Minimum year {} exceeds maximum year {}.",
            args.year_min, args.year_max
        )))
    }

    else if path_is_empty(&args.male_first_names_file) {
        Err(String::from(format!(
            "Male first names file not specified, and {} not set in environment.",
            ENV_MALE_FIRST_NAMES_FILE
        )))
    }

    else if path_is_empty(&args.female_first_names_file) {
        Err(String::from(format!(
            "Female first names file not specified, and {} not set in environment.",
            ENV_FEMALE_FIRST_NAMES_FILE
        )))
    }

    else if path_is_empty(&args.last_names_file) {
        Err(String::from(format!(
            "Last names file not specified, and {} is not set in environment.",
            ENV_LAST_NAMES_FILE
        )))
    }

    else {
        Ok(args)
    }
}
