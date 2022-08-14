use crate::TakeValue::*;
use clap::{App, Arg};
use once_cell::sync::OnceCell;
use regex::Regex;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom};

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
    let num_files = config.files.len();

    for (num, filename) in config.files.iter().enumerate() {
        match File::open(&filename) {
            Err(e) => eprintln!("{}: {}", filename, e),
            Ok(file) => {
                if !config.quiet && num_files > 1 {
                    println!("{}==> {} <==", if num > 0 { "\n" } else { "" }, filename);
                }

                let file = BufReader::new(file);
                let (total_lines, total_bytes) = count_lines_bytes(&filename)?;

                if let Some(num_bytes) = &config.bytes {
                    print_bytes(file, num_bytes, total_bytes)?;
                } else {
                    print_lines(file, &config.lines, total_lines)?;
                }
            }
        }
    }

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

fn count_lines_bytes(filename: &str) -> MyResult<(i64, i64)> {
    let mut file = BufReader::new(File::open(filename)?);
    let mut lines = 0;
    let mut bytes = 0;
    let mut buf = String::new();

    loop {
        let bytes_read = file.read_line(&mut buf)?;

        if bytes_read == 0 {
            break;
        }

        lines += 1;
        bytes += bytes_read as i64;

        buf.clear();
    }

    Ok((lines as i64, bytes as i64))
}

fn print_lines(mut file: impl BufRead, num_lines: &TakeValue, total_lines: i64) -> MyResult<()> {
    if let Some(start) = get_start_index(num_lines, total_lines) {
        let mut line_num = 0;
        let mut buf = Vec::new();

        loop {
            let bytes_read = file.read_until(b'\n', &mut buf)?;

            if bytes_read == 0 {
                break;
            }

            if line_num >= start {
                print!("{}", String::from_utf8_lossy(&buf));
            }

            line_num += 1;
            buf.clear();
        }
    }

    Ok(())
}

fn print_bytes<T: Read + Seek>(
    mut file: T,
    num_bytes: &TakeValue,
    total_bytes: i64,
) -> MyResult<()> {
    if let Some(start) = get_start_index(num_bytes, total_bytes) {
        file.seek(SeekFrom::Start(start))?;

        let mut buf = Vec::new();

        file.read_to_end(&mut buf)?;

        if !buf.is_empty() {
            print!("{}", String::from_utf8_lossy(&buf));
        }
    }

    Ok(())
}

fn get_start_index(take_val: &TakeValue, total: i64) -> Option<u64> {
    match take_val {
        &TakeNum(n) if n > 0 && n > total => None,
        &TakeNum(n) if n < 0 && n.abs() > total => Some(0),
        &TakeNum(n) if n > 0 => Some((n - 1) as u64),
        &TakeNum(n) if n < 0 => Some((total + n) as u64),
        &TakeNum(_) => None,
        &PlusZero => {
            if total == 0 {
                None
            } else {
                Some(0)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{count_lines_bytes, get_start_index, parse_num, TakeValue::*};

    #[test]
    fn test_get_start_index() {
        // +0 from an empty file (0 lines/bytes) returns None
        assert_eq!(get_start_index(&PlusZero, 0), None);

        // +0 from a nonempty file returns an index that
        // is one less than the number of lines/bytes
        assert_eq!(get_start_index(&PlusZero, 1), Some(0));

        // Taking 0 lines/bytes returns None
        assert_eq!(get_start_index(&TakeNum(0), 1), None);

        // Taking any lines/bytes from an empty file returns None
        assert_eq!(get_start_index(&TakeNum(1), 0), None);

        // Taking more lines/bytes than is available returns None
        assert_eq!(get_start_index(&TakeNum(2), 1), None);

        // When starting line/byte is less than total lines/bytes,
        // return one less than starting number
        assert_eq!(get_start_index(&TakeNum(1), 10), Some(0));
        assert_eq!(get_start_index(&TakeNum(2), 10), Some(1));
        assert_eq!(get_start_index(&TakeNum(3), 10), Some(2));

        // When starting line/byte is negative and less than total,
        // return total - start
        assert_eq!(get_start_index(&TakeNum(-1), 10), Some(9));
        assert_eq!(get_start_index(&TakeNum(-2), 10), Some(8));
        assert_eq!(get_start_index(&TakeNum(-3), 10), Some(7));

        // When the starting line/byte is negative and more than the total,
        // return 0 to print the whole file
        assert_eq!(get_start_index(&TakeNum(-20), 10), Some(0));
    }

    #[test]
    fn test_count_lines_bytes() {
        let res = count_lines_bytes("tests/inputs/one.txt");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), (1, 24));

        let res = count_lines_bytes("tests/inputs/ten.txt");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), (10, 49));
    }

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
