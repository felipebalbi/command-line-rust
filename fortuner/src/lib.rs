use clap::{App, Arg};
use regex::{Regex, RegexBuilder};
use std::error::Error;

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    sources: Vec<String>,
    pattern: Option<Regex>,
    seed: Option<u64>,
}

pub fn get_args() -> MyResult<Config> {
    let matches = App::new("fortuner")
        .version("0.1.0")
        .author("Felipe Balbi <felipe@balbi.sh>")
        .about("Rust fortune")
        .arg(
            Arg::with_name("files")
                .value_name("FILE")
                .help("Input files or directories")
                .required(true)
                .multiple(true),
        )
        .arg(
            Arg::with_name("pattern")
                .value_name("PATTERN")
                .short("m")
                .long("pattern")
                .help("Pattern")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("seed")
                .value_name("SEED")
                .short("s")
                .long("seed")
                .help("Random seed")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("insensitive")
                .short("i")
                .long("insensitive")
                .help("Case-insensitive pattern matching"),
        )
        .get_matches();

    let sources = matches.values_of_lossy("files").unwrap();

    let pattern = matches
        .value_of("pattern")
        .map(|p| {
            RegexBuilder::new(p)
                .case_insensitive(matches.is_present("insensitive"))
                .build()
                .map_err(|_| format!("Invalid pattern \"{}\"", p))
        })
        .transpose()?;

    let seed = matches.value_of("seed").map(parse_u64).transpose()?;

    Ok(Config {
        sources,
        pattern,
        seed,
    })
}

pub fn run(config: Config) -> MyResult<()> {
    println!("{:#?}", config);

    Ok(())
}

fn parse_u64(val: &str) -> MyResult<u64> {
    val.parse()
        .map_err(|_| format!("\"{}\" not a valid integer", val).into())
}

#[cfg(test)]
mod tests {
    use super::parse_u64;
    use std::path::PathBuf;

    #[test]
    fn test_parse_u64() {
        let res = parse_u64("a");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "\"a\" not a valid integer");

        let res = parse_u64("0");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 0);

        let res = parse_u64("4");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 4);
    }
}
