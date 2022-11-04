use std::fs::File;
use std::path::PathBuf;
use std::io::{self, prelude::*};
use chrono::naive::{NaiveDate, NaiveDateTime};
use rand::Rng;
use rand::seq::SliceRandom;
use csv::{ReaderBuilder, StringRecordsIter};
use tailcall::tailcall;
use crate::path::path_str;
use crate::args::Arguments;

#[derive(PartialEq)]
pub enum Gender {
    Male,
    Female
}

impl Gender {
    pub fn to_str(&self) -> &str {
        if *self == Gender::Male {
            "M"
         }
         else {
            "F"
         }
    }
}

pub struct Person {
    pub first_name: String,
    pub middle_name: String,
    pub last_name: String,
    pub gender: Gender,
    pub birth_date: NaiveDate,
    pub ssn: String
}

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

fn make_ssn(ssn_prefixes: &Vec<u32>) -> String {
    let mut rng = rand::thread_rng();
    let first_index = rng.gen_range(0..ssn_prefixes.len());
    let first = ssn_prefixes[first_index];
    let second = rng.gen_range(10..=99);
    let third = rng.gen_range(1000..=9999);

    format!("{}-{}-{}", first, second, third)
}

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

pub fn read_last_names(path: &PathBuf) -> Result<Vec<String>, String> {
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

pub fn read_first_names(path: &PathBuf) -> Result<(Vec<String>, Vec<String>), String> {
    #[tailcall]
    fn load_next(mut it: StringRecordsIter<File>,
                 path: &PathBuf,
                 line_num: u32,
                 mut f_buf: Vec<String>,
                 mut m_buf: Vec<String>) -> Result<(Vec<String>, Vec<String>), String> {
        match it.next() {
            Some(Err(e)) => Err(format!("{}", e)),
            Some(Ok(v)) if v.len() != 2 => {
                Err(format!("\"{}\", line {}: Expected 2 fields, not {}",
                            path_str(&path), line_num, v.len()))
            },
            Some(Ok(v)) if (v[0].trim() == "") || (v[1].trim() == "") => {
                Err(format!(
                    "\"{}\", line {}: Unexpected empty field(s)",
                    path_str(path), line_num
                ))
            },
            Some(Ok(v)) if v[1].to_lowercase() == "f" => {
                f_buf.push(v[0].to_string());
                load_next(it, path, line_num + 1, f_buf, m_buf)
            },
            Some(Ok(v)) if v[1].to_lowercase() == "m" => {
                m_buf.push(v[0].to_string());
                load_next(it, path, line_num + 1, f_buf, m_buf)
            },
            Some(Ok(v)) => Err(format!(
                "\"{}\", line {}: Unknown gender: {}",
                path_str(path), line_num, v[1].to_string()
            )),
            None => Ok((f_buf, m_buf))
        }
    }

    let mut r = ReaderBuilder::new()
        .has_headers(false)
        .from_path(path)
        .map_err(|e| format!("\"{}\": {}", path_str(path), e))?;

    let f_buf: Vec<String> = Vec::new();
    let m_buf: Vec<String> = Vec::new();
    let it = r.records();

    load_next(it, path, 1, f_buf, m_buf)
}
