use std::ffi::OsStr;
use std::env;
use std::path::PathBuf;
use std::collections::HashMap;
use clap::{Command, Arg, ArgAction};
use chrono::{Duration, Utc, Datelike};
use crate::numlib::parse_int;

const STARTING_YEAR_DEFAULT_DELTA: u32 = 90;
const ENDING_YEAR_DEFAULT_DELTA: u32 = 18;
const ENV_FIRST_NAMES_FILE: &str = "PEOPLEGEN_FIRST_NAMES";
const ENV_LAST_NAMES_FILE: &str = "PEOPLEGEN_LAST_NAMES";

#[derive(Debug, Copy, Clone)]
pub enum HeaderFormat {
    SnakeCase,
    CamelCase,
    HumanReadable
}

// Command-line arguments, as parsed.
#[derive(Debug)]
pub struct Arguments {
    female_percent: u32,
    male_percent: u32,
    generate_ssns: bool,
    generate_ids: bool,
    header_format: HeaderFormat,
    year_min: u32,
    year_max: u32,
    verbose: bool,
    first_names_file: PathBuf,
    last_names_file: PathBuf,
    output_file: PathBuf
}

/// Parse the command line arguments into an `Arguments` structure.
/// Returns an `Ok` with the parsed arguments, or an `Err` with a message
/// on error.
pub fn parse_args() -> Result<Arguments, String> {
    let header_format_map: HashMap<&str, HeaderFormat> = HashMap::from([
        ("snake", HeaderFormat::SnakeCase),
        ("nice", HeaderFormat::HumanReadable),
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
    let first_names_default = getenv(ENV_FIRST_NAMES_FILE);
    let last_names_default = getenv(ENV_LAST_NAMES_FILE);

    let parser = Command::new("peoplegen")
        .version("0.1.0")
        .author("bmc@clapper.org")
        .about("Generate fake people data in a CSV")
        .arg(Arg::new("female")
                 .short('f')
                 .long("female")
                 .default_value("50")
                 .value_name("PERCENT")
                 .help("Percentage of female names."))
        .arg(Arg::new("male")
                 .short('m')
                 .long("male")
                 .default_value("50")
                 .value_name("PERCENT")
                 .help("Percentage of male names."))
        .arg(Arg::new("first-names")
                 .short('F')
                 .long("first-names")
                 .value_name("PATH")
                 .help(format!(
"Path to CSV file containing first names and genders. The first
column must be the name, and the second is the gender (currently
'F' or 'M'). The file is assumed NOT to have a header. If not specified,
defaults to the value of environment variable {}.", ENV_FIRST_NAMES_FILE)))
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
                 .help("Generate fake Social Security numbers"))
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
        .arg(Arg::new("verbose")
                 .short('v')
                 .long("verbose")
                 .action(ArgAction::SetTrue)
                 .help("Enable verbose processing messages"))
        .arg(Arg::new("output")
                 .required(true)
                 .value_name("OUTPUT_FILE")
                 .help("Path to output file"));

    let matches = parser.get_matches();

    // NOTE: It's okay to use unwrap() rather than unwrap_or() on arguments
    // with a default, because they'll never come back as None.
    let female_percent = matches
        .get_one::<String>("female")
        .map(|s| parse_int(s))
        .unwrap()?;
    let male_percent = matches
        .get_one::<String>("male")
        .map(|s| parse_int(s))
        .unwrap()?;
    let year_min = matches
        .get_one::<u32>("year-min")
        // In this case, get_one() actually returns a reference to a u32.
        // We can use map() to dereference it.
        .map(|reference| *reference)
        .unwrap_or_else(|| year_before_now(ENDING_YEAR_DEFAULT_DELTA));
    let year_max = matches
        .get_one::<u32>("year-max")
        .map(|reference| *reference)
        .unwrap_or_else(|| year_before_now(STARTING_YEAR_DEFAULT_DELTA));
    let header_format = matches
        .get_one::<String>("header-format")
        .map(|s| parse_header_format(s))
        .unwrap()?;
    let output_file = matches
        .get_one::<String>("output")
        .map(PathBuf::from)
        .unwrap();
    let first_names_file = matches
         .get_one::<String>("first-names")
         .unwrap_or(&first_names_default);
    let last_names_file = matches
         .get_one::<String>("last-names")
         .unwrap_or(&last_names_default);

    validate(Arguments {
        female_percent,
        male_percent,
        generate_ssns: *matches.get_one::<bool>("ssn").unwrap(),
        generate_ids: *matches.get_one::<bool>("id").unwrap(),
        header_format,
        year_min,
        year_max,
        first_names_file: PathBuf::from(first_names_file),
        last_names_file: PathBuf::from(last_names_file),
        verbose: *matches.get_one::<bool>("verbose").unwrap(),
        output_file: output_file
    })
}

fn getenv(s: &str) -> String {
    // match env::var_os(s) {
    //     Some(v) => v.into_string().unwrap(),
    //     None => String::new()
    // }

    env::var_os(s)
        .map(|v| v.into_string().unwrap())
        .unwrap_or_else(|| String::new())
}

fn year_before_now(years: u32) -> u32 {
    // There's no Duration::years(), so just use weeks and multiply.
    let y = years as i64;
    (Utc::now() - Duration::weeks(y * 52)).year() as u32
}

fn path_is_empty(p: &PathBuf) -> bool {
    p.as_path().as_os_str() == ""
}

/// Cross-validate the parsed arguments.
fn validate(args: Arguments) -> Result<Arguments, String> {
    fn file_ext(p: &PathBuf) -> &str {
        p.extension().and_then(OsStr::to_str).unwrap_or("")
    }

    if (args.female_percent + args.male_percent) != 100 {
        Err(String::from("Female and male percentages must add up to 100."))
    }

    else if file_ext(&args.output_file) != "csv" {
        Err(String::from(format!(
            "Output path \"{}\" does not have required \".csv\" extension.",
            args.output_file.display()
        )))
    }

    else if args.year_min > args.year_max {
        Err(String::from(format!(
            "Minimum year {} exceeds maximum year {}.",
            args.year_min, args.year_max
        )))
    }

    else if path_is_empty(&args.first_names_file) {
        Err(String::from(format!(
            "First names file not specified, and {} is not set in environment.",
            ENV_FIRST_NAMES_FILE
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
