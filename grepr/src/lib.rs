use clap::{App, Arg};
use regex::{Regex, RegexBuilder};
use std::error::Error;
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader};
use walkdir::WalkDir;

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
    let entries = find_files(&config.files, config.recursive);
    let num_files = entries.len();

    let print = |fname: &str, val: &str| {
        if num_files > 1 {
            print!("{}:{}", fname, val);
        } else {
            print!("{}", val);
        }
    };

    for entry in entries {
        match entry {
            Err(e) => eprintln!("{}", e),
            Ok(filename) => match open(&filename) {
                Err(e) => eprintln!("{}: {}", filename, e),
                Ok(file) => match find_lines(file, &config.pattern, config.invert_match) {
                    Err(e) => eprintln!("{}", e),
                    Ok(lines) => {
                        if config.count {
                            print(&filename, &format!("{}\n", lines.len()));
                        } else {
                            for line in &lines {
                                print(&filename, line);
                            }
                        }
                    }
                },
            },
        }
    }

    Ok(())
}

fn find_files(paths: &[String], recursive: bool) -> Vec<MyResult<String>> {
    let mut files = vec![];

    for path in paths {
        match path.as_str() {
            "-" => files.push(Ok(path.to_string())),
            _ => match fs::metadata(path) {
                Ok(metadata) => {
                    if metadata.is_dir() {
                        if !recursive {
                            files.push(Err(From::from(format!("{} is a directory", path))));
                        } else {
                            for entry in WalkDir::new(path)
                                .into_iter()
                                .flatten()
                                .filter(|e| e.file_type().is_file())
                            {
                                files.push(Ok(entry.path().display().to_string()));
                            }
                        }
                    } else if metadata.is_file() {
                        files.push(Ok(path.to_string()));
                    }
                }
                Err(e) => files.push(Err(From::from(format!("{}: {}", path, e)))),
            },
        }
    }

    files
}

fn find_lines<T: BufRead>(
    mut file: T,
    pattern: &Regex,
    invert_match: bool,
) -> MyResult<Vec<String>> {
    let mut line = String::new();
    let mut lines = vec![];

    loop {
        let bytes = file.read_line(&mut line)?;

        if bytes == 0 {
            break;
        }

        if invert_match ^ pattern.is_match(line.as_str()) {
            lines.push(line.clone());
        }

        line.clear();
    }

    Ok(lines)
}

fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}

#[cfg(test)]
mod tests {
    use super::{find_files, find_lines};
    use rand::{distributions::Alphanumeric, Rng};
    use regex::{Regex, RegexBuilder};
    use std::io::Cursor;

    #[test]
    fn test_find_files() {
        // Verify that the function finds a file known to exist
        let files = find_files(&["./tests/inputs/fox.txt".to_string()], false);
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].as_ref().unwrap(), "./tests/inputs/fox.txt");

        // The function should reject a directory without the recursive option
        let files = find_files(&["./tests/inputs".to_string()], false);
        assert_eq!(files.len(), 1);
        if let Err(e) = &files[0] {
            assert_eq!(e.to_string(), "./tests/inputs is a directory");
        }

        // Verify the function recurses to find four files in the directory
        let res = find_files(&["./tests/inputs".to_string()], true);
        let mut files: Vec<String> = res
            .iter()
            .map(|r| r.as_ref().unwrap().replace("\\", "/"))
            .collect();
        files.sort();
        assert_eq!(files.len(), 4);
        assert_eq!(
            files,
            vec![
                "./tests/inputs/bustle.txt",
                "./tests/inputs/empty.txt",
                "./tests/inputs/fox.txt",
                "./tests/inputs/nobody.txt",
            ]
        );

        // Generate a random string to represent a nonexistent file
        let bad: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(7)
            .map(char::from)
            .collect();

        // Verify that the function returns the bad file as an error
        let files = find_files(&[bad], false);
        assert_eq!(files.len(), 1);
        assert!(files[0].is_err());
    }

    #[test]
    fn test_find_lines() {
        let text = b"Lorem\nIpsum\r\nDOLOR";

        // The pattern _or_ should match the one line, "Lorem"
        let re1 = Regex::new("or").unwrap();
        let matches = find_lines(Cursor::new(&text), &re1, false);
        assert!(matches.is_ok());
        assert_eq!(matches.unwrap().len(), 1);

        // When inverted, the function should match the other two lines
        let matches = find_lines(Cursor::new(&text), &re1, true);
        assert!(matches.is_ok());
        assert_eq!(matches.unwrap().len(), 2);

        // This regex will be case-insensitive
        let re2 = RegexBuilder::new("or")
            .case_insensitive(true)
            .build()
            .unwrap();

        // The two lines "Lorem" and "DOLOR" should match
        let matches = find_lines(Cursor::new(&text), &re2, false);
        assert!(matches.is_ok());
        assert_eq!(matches.unwrap().len(), 2);

        // When inverted, the one remaining line should match
        let matches = find_lines(Cursor::new(&text), &re2, true);
        assert!(matches.is_ok());
        assert_eq!(matches.unwrap().len(), 1);
    }
}
