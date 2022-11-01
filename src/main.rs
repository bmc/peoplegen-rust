//use std::io;
use std::process;
use std::collections::HashMap;
use clap::{Command,Arg,ArgAction};
use chrono::{Duration, Utc, Datelike};
use crate::numlib::parse_int;
use crate::path::file_extension;

pub mod numlib;
pub mod path;

#[derive(Debug, Copy, Clone)]
enum HeaderFormat {
    SnakeCase,
    CamelCase,
    HumanReadable
}

#[derive(Debug)]
struct Arguments {
    female_percent: u32,
    male_percent: u32,
    generate_ssns: bool,
    generate_ids: bool,
    header_format: HeaderFormat,
    year_min: u32,
    year_max: u32,
    verbose: bool,
    output_file: String
}

const STARTING_YEAR_DEFAULT_DELTA: u32 = 90;
const ENDING_YEAR_DEFAULT_DELTA: u32 = 18;

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

fn parse_args() -> Result<Arguments, String> {
    let HEADER_FORMAT_MAP: HashMap<String, HeaderFormat> = HashMap::from([
        (String::from("snake"), HeaderFormat::SnakeCase),
        (String::from("nice"), HeaderFormat::HumanReadable),
        (String::from("camel"), HeaderFormat::CamelCase),
    ]);
    // See https://stackoverflow.com/a/56724224/53495
    let HEADER_FORMATS: Vec<String> = HEADER_FORMAT_MAP.keys().cloned().collect();

    fn validate(args: Arguments) -> Result<Arguments, String> {
        if (args.female_percent + args.male_percent) != 100 {
            Err(String::from("Female and male percentages must add up to 100."))
        }
        else if (file_extension(&args.output_file).unwrap_or("") != "csv") {
            Err(String::from(format!(
                "Output path \"{}\" does not have required \".csv\" extension.",
                args.output_file
            )))
        }
        else {
            Ok(args)
        }
    }

    fn year_before_now(years: u32) -> u32 {
        // There's no Duration::years(), so just use weeks and multiply.
        let y = years as i64;
        (Utc::now() - Duration::weeks(y * 52)).year() as u32
    }

    let parse_header_format = |s: &String| {
        HEADER_FORMAT_MAP
          .get(s)
          .map_or(Err(format!("Bad header format value: \"{s}\"")), |h| Ok(*h))
    };

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
                               HEADER_FORMATS.join(", "))))
        .arg(Arg::new("year-min")
                 .short('y')
                 .long("year-min")
                 .help(format!("The minimum year for birth dates. Defaults to
{} years ago from this year.", STARTING_YEAR_DEFAULT_DELTA)))
        .arg(Arg::new("year-max")
                 .short('Y')
                 .long("year-max")
                 .help(format!("The maximum year for birth dates. Defaults to
{} years ago from this year.", ENDING_YEAR_DEFAULT_DELTA)))
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
        .get_one::<String>("year-min")
        .map(|s| parse_int(s))
        .unwrap_or_else(|| Ok(year_before_now(STARTING_YEAR_DEFAULT_DELTA)))?;
    let year_max = matches
        .get_one::<String>("year-max")
        .map(|s| parse_int(s))
        .unwrap_or_else(|| Ok(year_before_now(STARTING_YEAR_DEFAULT_DELTA)))?;
    let header_format = matches
        .get_one::<String>("header-format")
        .map(|s| parse_header_format(s))
        .unwrap()?;

    validate(Arguments {
        female_percent,
        male_percent,
        generate_ssns: *matches.get_one::<bool>("ssn").unwrap(),
        generate_ids: *matches.get_one::<bool>("id").unwrap(),
        header_format,
        year_min,
        year_max,
        verbose: *matches.get_one::<bool>("verbose").unwrap(),
        output_file: matches.get_one::<String>("output").unwrap().to_string()
    })
}
