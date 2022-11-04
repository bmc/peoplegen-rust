use std::process;
use std::path::PathBuf;
use csv::WriterBuilder;
use crate::args::{Arguments, HeaderFormat, parse_args};
use crate::people::{Person, read_names_file, make_people};

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
    read_names_file(&args.male_first_names_file)

    .and_then(|(male_first_names)| {
        read_names_file(&args.female_first_names_file)
            .map(|female_first_names| (female_first_names, male_first_names))
    })

    .and_then(|(female_first_names, male_first_names)| {
        read_names_file(&args.last_names_file)
            .map(|last_names| (female_first_names, male_first_names, last_names))
    })

    .and_then(|(female_first_names, male_first_names, last_names)| {
      let people = make_people(
          &args,
          &male_first_names,
          &female_first_names,
          &last_names
      );
      Ok(people)
    })

    .and_then(|people| {
      assert!(people.len() == (args.total as usize));
      write_people(&args.output_file, &args, &people)
    })

    .and_then(|total| {
        println!("Wrote {} records(s) to \"{}\".",
                 total, args.output_file.display());
        Ok(())
    })
}

fn write_people(path: &PathBuf,
                args: &Arguments,
                people: &Vec<Person>) -> Result<usize, String> {
    let mut w = WriterBuilder::new()
        .from_path(path)
        .map_err(|e| format!("{}", e))?;

    let (id_header, base_headers, ssn_header) = match args.header_format {
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

    if args.generate_ids {
        header.push(id_header);
    }

    header.extend(base_headers);

    if args.generate_ssns {
        header.push(ssn_header);
    }

    w.write_record(&header).map_err(|e| format!("{}", e))?;
    for (i, p) in people.iter().enumerate() {
        let id = i + 1;
        let id_str = id.to_string();
        let mut rec: Vec<&String> = Vec::new();
        if args.generate_ids {
            rec.push(&id_str);
        }
        let birth_str = p.birth_date.format("%Y-%m-%d").to_string();
        let gender_str = p.gender.to_str().to_string();
        rec.extend([
            &p.first_name, &p.middle_name, &p.last_name,
            &gender_str, &birth_str
        ]);

        if args.generate_ssns {
            rec.push(&p.ssn);
        }

        w.write_record(&rec).map_err(|e| format!("{}", e))?;
    }
    Ok(people.len())
}



