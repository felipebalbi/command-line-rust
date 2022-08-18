use chrono::{Datelike, Local, NaiveDate};
use clap::{App, Arg};
use std::error::Error;
use std::str::FromStr;

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    month: Option<u32>,
    year: i32,
    today: NaiveDate,
}

const MONTH_NAMES: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

pub fn get_args() -> MyResult<Config> {
    let matches = App::new("calr")
        .version("0.1.")
        .author("Felipe Balbi <felipe@balbi.sh")
        .about("Rust cal")
        .arg(Arg::with_name("year").value_name("YEAR").takes_value(true))
        .arg(
            Arg::with_name("month")
                .value_name("MONTH")
                .short("m")
                .help("Month name or number (1-12)")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("show_year")
                .help("Show whole current year")
                .short("y")
                .long("year")
                .conflicts_with_all(&["month", "year"]),
        )
        .get_matches();

    let today = Local::today();

    let mut month = matches.value_of("month").map(parse_month).transpose()?;
    let mut year = matches.value_of("year").map(parse_year).transpose()?;

    let today = Local::today();
    if matches.is_present("show_year") {
        month = None;
        year = Some(today.year());
    } else if month.is_none() && year.is_none() {
        month = Some(today.month());
        year = Some(today.year());
    }

    let year = year.unwrap_or_else(|| today.year());
    let today = today.naive_local();

    Ok(Config { month, year, today })
}

pub fn run(config: Config) -> MyResult<()> {
    println!("{:?}", config);

    Ok(())
}

fn parse_int<T: FromStr>(val: &str) -> MyResult<T> {
    val.parse()
        .map_err(|_| format!("Invalid integer \"{}\"", val).into())
}

fn parse_year(year: &str) -> MyResult<i32> {
    parse_int(year).and_then(|y| {
        if (1..=9999).contains(&y) {
            Ok(y)
        } else {
            Err(format!("year \"{}\" not in the range 1 through 9999", year).into())
        }
    })
}

fn parse_month(month: &str) -> MyResult<u32> {
    match parse_int(month) {
        // A number was passed in as a string
        Ok(m) => {
            if (1..=12).contains(&m) {
                Ok(m)
            } else {
                Err(format!("month \"{}\" not in the range 1 through 12", month).into())
            }
        }

        // Maybe the name of the month of was passed in
        _ => {
            let lower = &month.to_lowercase();

            let matches: Vec<_> = MONTH_NAMES
                .iter()
                .enumerate()
                .filter_map(|(i, name)| {
                    if name.to_lowercase().starts_with(lower) {
                        Some(i + 1)
                    } else {
                        None
                    }
                })
                .collect();

            if matches.len() == 1 {
                // There was a single match, we're good. Just return it
                Ok(matches[0] as u32)
            } else {
                // Oops, can't decide which month
                Err(format!("Invalid month \"{}\"", month).into())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{parse_int, parse_month, parse_year};

    #[test]
    fn test_parse_int() {
        // Parse positive int as usize
        let res = parse_int::<usize>("1");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 1usize);

        // Parse negative int as i32
        let res = parse_int::<i32>("-1");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), -1i32);

        // Fail on a string
        let res = parse_int::<i64>("foo");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "Invalid integer \"foo\"");
    }

    #[test]
    fn test_parse_year() {
        let res = parse_year("1");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 1i32);

        let res = parse_year("9999");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 9999i32);

        let res = parse_year("0");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "year \"0\" not in the range 1 through 9999"
        );

        let res = parse_year("10000");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "year \"10000\" not in the range 1 through 9999"
        );

        let res = parse_year("foo");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "Invalid integer \"foo\"");
    }

    #[test]
    fn test_parse_month() {
        let res = parse_month("1");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 1u32);

        let res = parse_month("12");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 12u32);

        let res = parse_month("jan");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 1u32);

        let res = parse_month("0");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "month \"0\" not in the range 1 through 12"
        );

        let res = parse_month("13");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "month \"13\" not in the range 1 through 12"
        );

        let res = parse_month("foo");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "Invalid month \"foo\"");
    }
}
