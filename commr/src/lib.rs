use crate::Column::*;
use clap::{App, Arg};
use std::cmp::Ordering::*;
use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader};

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    file1: String,
    file2: String,
    show_col1: bool,
    show_col2: bool,
    show_col3: bool,
    insensitive: bool,
    delimiter: String,
}

pub fn get_args() -> MyResult<Config> {
    let matches = App::new("commr")
        .version("0.1.0")
        .author("Felipe Balbi")
        .about("Rust comm")
        .arg(
            Arg::with_name("file1")
                .value_name("FILE1")
                .help("Input file 1")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("file2")
                .value_name("FILE2")
                .help("Input file 2")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("insensitive")
                .short("i")
                .help("Case-insensitive comparison of lines"),
        )
        .arg(
            Arg::with_name("suppress_col1")
                .short("1")
                .help("Suppress printing of column 1"),
        )
        .arg(
            Arg::with_name("suppress_col2")
                .short("2")
                .help("Suppress printing of column 2"),
        )
        .arg(
            Arg::with_name("suppress_col3")
                .short("3")
                .help("Suppress printing of column 3"),
        )
        .arg(
            Arg::with_name("delimiter")
                .value_name("DELIM")
                .short("d")
                .long("output-delimiter")
                .help("Output delimiter")
                .takes_value(true)
                .default_value("\t"),
        )
        .get_matches();

    let file1 = matches.value_of("file1").unwrap().to_string();
    let file2 = matches.value_of("file2").unwrap().to_string();
    let show_col1 = !matches.is_present("suppress_col1");
    let show_col2 = !matches.is_present("suppress_col2");
    let show_col3 = !matches.is_present("suppress_col3");
    let insensitive = matches.is_present("insensitive");
    let delimiter = matches.value_of("delimiter").unwrap().to_string();

    Ok(Config {
        file1,
        file2,
        show_col1,
        show_col2,
        show_col3,
        insensitive,
        delimiter,
    })
}

enum Column<'a> {
    Column1(&'a str),
    Column2(&'a str),
    Column3(&'a str),
}

pub fn run(config: Config) -> MyResult<()> {
    let file1 = &config.file1;
    let file2 = &config.file2;

    if file1 == "-" && file2 == "-" {
        return Err(From::from("Both input files cannot be STDIN (\"-\")"));
    }

    let case = |line: String| {
        if config.insensitive {
            line.to_lowercase()
        } else {
            line
        }
    };

    let printer = |col: Column| {
        let mut output = vec![];

        match col {
            Column1(l) => {
                if config.show_col1 {
                    output.push(l);
                }
            }
            Column2(l) => {
                if config.show_col2 {
                    if config.show_col1 {
                        output.push("");
                    }

                    output.push(l);
                }
            }
            Column3(l) => {
                if config.show_col3 {
                    if config.show_col1 {
                        output.push("");
                    }

                    if config.show_col3 {
                        output.push("");
                    }

                    output.push(l);
                }
            }
        }

        if !output.is_empty() {
            println!("{}", output.join(&config.delimiter));
        }
    };

    let mut lines1 = open(file1)?.lines().filter_map(Result::ok).map(case);
    let mut lines2 = open(file2)?.lines().filter_map(Result::ok).map(case);

    let mut line1 = lines1.next();
    let mut line2 = lines2.next();

    while line1.is_some() || line2.is_some() {
        match (&line1, &line2) {
            (Some(l1), Some(l2)) => match l1.cmp(l2) {
                Equal => {
                    printer(Column3(l1));
                    line1 = lines1.next();
                    line2 = lines2.next();
                }
                Less => {
                    printer(Column1(l1));
                    line1 = lines1.next();
                }
                Greater => {
                    printer(Column2(l2));
                    line2 = lines2.next();
                }
            },
            (Some(l1), None) => {
                printer(Column1(l1));
                line1 = lines1.next();
            }
            (None, Some(l2)) => {
                printer(Column2(l2));
                line2 = lines2.next();
            }
            _ => (),
        }
    }

    Ok(())
}

fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(
            File::open(filename).map_err(|e| format!("{}: {}", filename, e))?,
        ))),
    }
}
