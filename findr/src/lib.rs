use crate::EntryType::*;
use clap::{App, Arg};
use regex::Regex;
use std::error::Error;
use walkdir::WalkDir;

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug, Eq, PartialEq)]
enum EntryType {
    Dir,
    File,
    Link,
}

#[derive(Debug)]
pub struct Config {
    paths: Vec<String>,
    names: Vec<Regex>,
    entry_types: Vec<EntryType>,
}

pub fn get_args() -> MyResult<Config> {
    let matches = App::new("findr")
        .version("0.1.0")
        .author("Felipe Balbi <felipe@balbi.sh>")
        .about("Rust find")
        .arg(
            Arg::with_name("paths")
                .value_name("PATH")
                .help("Paths to search")
                .default_value(".")
                .takes_value(true)
                .multiple(true),
        )
        .arg(
            Arg::with_name("names")
                .short("n")
                .long("name")
                .value_name("NAME")
                .help("Name or Regex to match")
                .takes_value(true)
                .multiple(true),
        )
        .arg(
            Arg::with_name("types")
                .short("t")
                .long("type")
                .value_name("TYPE")
                .help("Type to filter")
                .takes_value(true)
                .possible_values(&["f", "l", "d"])
                .multiple(true),
        )
        .get_matches();

    let paths = matches.values_of_lossy("paths").unwrap();

    let names = matches
        .values_of_lossy("names")
        .map(|vals| {
            vals.into_iter()
                .map(|name| Regex::new(&name).map_err(|_| format!("Invalid --name \"{}\"", name)))
                .collect::<Result<Vec<_>, _>>()
        })
        .transpose()?
        .unwrap_or_default();

    let entry_types = matches
        .values_of_lossy("types")
        .map(|vals| {
            vals.iter()
                .map(|t| match t.as_str() {
                    "f" => File,
                    "l" => Link,
                    "d" => Dir,
                    _ => unreachable!("Invalid type"),
                })
                .collect()
        })
        .unwrap_or_default();

    Ok(Config {
        paths,
        names,
        entry_types,
    })
}

pub fn run(config: Config) -> MyResult<()> {
    for path in config.paths {
        for entry in WalkDir::new(path) {
            match entry {
                Err(e) => eprintln!("{}", e),
                Ok(entry) => println!("{}", entry.path().display()),
            }
        }
    }

    Ok(())
}