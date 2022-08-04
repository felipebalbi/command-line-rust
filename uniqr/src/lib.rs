use clap::{App, Arg};
use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    in_file: String,
    out_file: Option<String>,
    count: bool,
}

pub fn get_args() -> MyResult<Config> {
    let matches = App::new("uniqr")
        .version("0.1.0")
        .author("Felipe Balbi <felipe@balbi.sh>")
        .about("Rust uniq")
        .arg(
            Arg::with_name("in_file")
                .value_name("IN_FILE")
                .help("Input file")
                .takes_value(true)
                .default_value("-"),
        )
        .arg(
            Arg::with_name("out_file")
                .value_name("OUT_FILE")
                .help("Output file")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("count")
                .short("c")
                .long("count")
                .help("Prefix lines by the number of occurrences"),
        )
        .get_matches();

    let in_file = matches.value_of("in_file").unwrap().to_string();
    let out_file = matches.value_of("out_file").map(String::from);
    let count = matches.is_present("count");

    Ok(Config {
        in_file,
        out_file,
        count,
    })
}

pub fn run(config: Config) -> MyResult<()> {
    let mut file = open(&config.in_file).map_err(|e| format!("{}: {}", config.in_file, e))?;
    let mut line = String::new();
    let mut last = String::new();
    let mut out = create(config.out_file.as_deref())
        .map_err(|e| format!("{}: {}", config.out_file.unwrap(), e))?;

    let mut count: u64 = 0;

    loop {
        let bytes = file.read_line(&mut line)?;
        if bytes == 0 {
            break;
        }

        if line.trim_end() != last.trim_end() {
            write!(out, "{}{}", format_count(count, config.count), last)?;
            last = line.clone();
            count = 0;
        }

        count += 1;
        line.clear()
    }

    write!(out, "{}{}", format_count(count, config.count), last)?;

    Ok(())
}

fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}

fn create(filename: Option<&str>) -> MyResult<Box<dyn Write>> {
    match filename {
        Some(f) => Ok(Box::new(File::create(f)?)),
        _ => Ok(Box::new(io::stdout())),
    }
}

fn format_count(count: u64, show: bool) -> String {
    if count > 0 && show {
        format!("{:>4} ", count)
    } else {
        format!("{}", "")
    }
}
