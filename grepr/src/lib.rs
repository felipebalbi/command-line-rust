use clap::{App, Arg};
use regex::{Regex, RegexBuilder};
use std::error::Error;

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    pattern: Regex,
    files: Vec<String>,
    recursive: bool,
    count: bool,
    invert_match: bool,
}

pub fn get_args() -> MyResult<Config> {
    let matches = App::new("grepr")
        .version("0.1.0")
        .author("Felipe Balbi <felipe@balbi.sh>")
        .about("Rust grep")
        .arg(
            Arg::with_name("pattern")
                .value_name("PATERN")
                .help("Search pattern")
                .required(true),
        )
        .arg(
            Arg::with_name("files")
                .value_name("FILE")
                .help("Input file(s)")
                .default_value("-")
                .multiple(true),
        )
        .arg(
            Arg::with_name("count")
                .short("c")
                .long("count")
                .help("Count occurrences"),
        )
        .arg(
            Arg::with_name("insensitive")
                .short("i")
                .long("insensitive")
                .help("Case-insensitive"),
        )
        .arg(
            Arg::with_name("invert-match")
                .short("v")
                .long("invert-match")
                .help("Invert match"),
        )
        .arg(
            Arg::with_name("recursive")
                .short("r")
                .long("recursive")
                .help("Recursive search"),
        )
        .get_matches();

    let pattern = matches
        .value_of("pattern")
        .map(|p| {
            RegexBuilder::new(p)
                .case_insensitive(matches.is_present("insensitive"))
                .build()
                .map_err(|_| format!("Invalid pattern \"{}\"", p))
        })
        .transpose()?
        .unwrap();
    let files = matches.values_of_lossy("files").unwrap();
    let recursive = matches.is_present("recursive");
    let count = matches.is_present("count");
    let invert_match = matches.is_present("invert-match");

    Ok(Config {
        pattern,
        files,
        recursive,
        count,
        invert_match,
    })
}

pub fn run(config: Config) -> MyResult<()> {
    println!("{:#?}", config);

    Ok(())
}
