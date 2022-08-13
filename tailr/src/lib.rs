use crate::TakeValue::*;
use clap::{App, Arg};
use once_cell::sync::OnceCell;
use regex::Regex;
use std::error::Error;

type MyResult<T> = Result<T, Box<dyn Error>>;

static PLUS_OR_MINUS_RE: OnceCell<Regex> = OnceCell::new();
static PLUS_ZERO_RE: OnceCell<Regex> = OnceCell::new();

#[derive(Debug, PartialEq)]
enum TakeValue {
    PlusZero,
    TakeNum(i64),
}

#[derive(Debug)]
pub struct Config {
    files: Vec<String>,
    lines: TakeValue,
    bytes: Option<TakeValue>,
    quiet: bool,
}

pub fn get_args() -> MyResult<Config> {
    let matches = App::new("tailr")
        .version("0.1.0")
        .about("Rust tail")
        .author("Felipe Balbi <felipe@balbi.sh>")
        .arg(
            Arg::with_name("files")
                .value_name("FILE")
                .help("Input file(s)")
                .multiple(true)
                .required(true),
        )
        .arg(
            Arg::with_name("lines")
                .short("n")
                .long("lines")
                .help("Number of lines")
                .takes_value(true)
                .value_name("LINES")
                .default_value("10"),
        )
        .arg(
            Arg::with_name("bytes")
                .short("c")
                .long("bytes")
                .help("Number of bytes")
                .takes_value(true)
                .value_name("BYTES")
                .conflicts_with("lines"),
        )
        .arg(Arg::with_name("quiet").short("q").long("quiet"))
        .get_matches();

    let files = matches.values_of_lossy("files").unwrap();

    let lines = matches
        .value_of("lines")
        .map(parse_num)
        .transpose()
        .map_err(|e| format!("illegal line count -- {}", e))?
        .unwrap();

    let bytes = matches
        .value_of("bytes")
        .map(parse_num)
        .transpose()
        .map_err(|e| format!("illegal byte count -- {}", e))?;

    let quiet = matches.is_present("quiet");

    Ok(Config {
        files,
        lines,
        bytes,
        quiet,
    })
}

pub fn run(config: Config) -> MyResult<()> {
    println!("{:#?}", config);

    Ok(())
}

fn parse_num(val: &str) -> MyResult<TakeValue> {
    let plus_or_minus = PLUS_OR_MINUS_RE.get_or_init(|| Regex::new(r"^[+-][0-9]+$").unwrap());
    let plus_zero = PLUS_ZERO_RE.get_or_init(|| Regex::new(r"^[+]0$").unwrap());

    match val.parse::<i64>() {
        Ok(_) if plus_zero.is_match(val) => Ok(PlusZero),
        Ok(n) if plus_or_minus.is_match(val) => Ok(TakeNum(n)),
        Ok(n) => Ok(TakeNum(-n)),
        _ => Err(val.into()),
    }
}

#[cfg(test)]
mod tests {
    use super::{parse_num, TakeValue::*};

    #[test]
    fn test_parse_num() {
        // All integers should be interpreted as negative numbers
        let res = parse_num("3");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(-3));

        // A leading "+" should result in a positive number
        let res = parse_num("+3");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(3));

        // An explicit "-" value should result in a negative number
        let res = parse_num("-3");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(-3));

        // Zero is zero
        let res = parse_num("0");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(0));

        // Plus zero is special
        let res = parse_num("+0");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), PlusZero);

        // Test boundaries
        let res = parse_num(&i64::MAX.to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(i64::MIN + 1));

        let res = parse_num(&(i64::MIN + 1).to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(i64::MIN + 1));

        let res = parse_num(&format!("+{}", i64::MAX));
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(i64::MAX));

        let res = parse_num(&i64::MIN.to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(i64::MIN));

        // A floating-point value is invalid
        let res = parse_num("3.14");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "3.14");

        // Any non-integer string is invalid
        let res = parse_num("foo");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "foo");
    }
}
