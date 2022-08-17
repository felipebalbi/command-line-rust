use clap::{App, Arg};
use regex::{Regex, RegexBuilder};
use std::error::Error;
use std::ffi::OsStr;
use std::fs;
use std::path::PathBuf;
use walkdir::{DirEntry, WalkDir};

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
    let files = find_files(&config.sources)?;
    println!("{:#?}", files);

    Ok(())
}

fn parse_u64(val: &str) -> MyResult<u64> {
    val.parse()
        .map_err(|_| format!("\"{}\" not a valid integer", val).into())
}

fn find_files(paths: &[String]) -> MyResult<Vec<PathBuf>> {
    let mut files = Vec::new();

    let file_filter = |entry: &DirEntry| {
        entry.file_type().is_file() && entry.path().extension() != Some(OsStr::new("dat"))
    };

    for path in paths {
        match fs::metadata(path) {
            Err(e) => return Err(format!("{}: {}", path, e).into()),
            Ok(_) => files.extend(
                WalkDir::new(path)
                    .into_iter()
                    .filter_map(Result::ok)
                    .filter(file_filter)
                    .map(|entry| entry.path().into()),
            ),
        }
    }

    files.sort();
    files.dedup();
    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::{find_files, parse_u64};
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

    #[test]
    fn test_find_files() {
        // Verify that the function finds a file known to exist
        let res = find_files(&["./tests/inputs/jokes".to_string()]);
        assert!(res.is_ok());

        let files = res.unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(
            files.get(0).unwrap().to_string_lossy(),
            "./tests/inputs/jokes"
        );

        // Fails to find a bad file
        let res = find_files(&["/path/does/not/exist".to_string()]);
        assert!(res.is_err());

        // Finds all the input files, excludes ".dat"
        let res = find_files(&["./tests/inputs".to_string()]);
        assert!(res.is_ok());

        // Check number and order of files
        let files = res.unwrap();
        assert_eq!(files.len(), 5);
        let first = files.get(0).unwrap().display().to_string();
        assert!(first.contains("ascii-art"));
        let last = files.last().unwrap().display().to_string();
        assert!(last.contains("quotes"));

        // Test for multiple sources, path must be unique and sorted
        let res = find_files(&[
            "./tests/inputs/jokes".to_string(),
            "./tests/inputs/ascii-art".to_string(),
            "./tests/inputs/jokes".to_string(),
        ]);
        assert!(res.is_ok());

        let files = res.unwrap();
        assert_eq!(files.len(), 2);
        if let Some(filename) = files.first().unwrap().file_name() {
            assert_eq!(filename.to_string_lossy(), "ascii-art".to_string())
        }

        if let Some(filename) = files.last().unwrap().file_name() {
            assert_eq!(filename.to_string_lossy(), "jokes".to_string())
        }
    }
}
