use clap::{Arg, Command};
use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read};

#[derive(Debug)]
pub struct Config {
    files: Vec<String>,
    lines: usize,
    bytes: Option<usize>
}

type RunResult<T> = Result<T, Box<dyn Error>>;

pub fn get_args() -> RunResult<Config> {
    let matches = Command::new("headr")
        .version("0.1.0")
        .about("head in Rust")
        .arg(
            Arg::new("files")
            .value_name("FILE")
            .help("Input file(s)")
            .num_args(0..)
            .default_value("-")
        )
        .arg(
            Arg::new("bytes")
                .short('c')
                .long("bytes")
                .value_name("BYTES")
                .conflicts_with("lines")
                .help("Number of bytes")
        )
        .arg(
            Arg::new("lines")
                .short('n')
                .long("lines")
                .value_name("LINES")
                .default_value("10")
                .help("Number of lines")
        )
        .get_matches();
    
    let files = matches.get_many::<String>("files")
        .unwrap()
        .map(|s| s.clone())
        .collect();
    
    let lines = matches.get_one::<String>("lines")
        .map(|s| parse_positive_int(s.as_str()))
        .transpose()
        .map_err(|err|
            format!("illegal line count -- {err}")
        )?
        .unwrap();

    let bytes = matches.get_one::<String>("bytes")
        .map(|s| parse_positive_int(s.as_str()))
        .transpose()
        .map_err(|err|
            format!("illegal byte count -- {err}")
        )?;

    Ok(Config {
        files,
        lines,
        bytes
    })
}

pub fn run(config: Config) -> RunResult<()> {
let num_files = config.files.len();

    for (file_num, filename) in config.files.iter().enumerate() {
        let mut file = match open(&filename) {
            Ok(file) => file,
            Err(err) => {
                eprintln!("{filename}: {err}");
                continue;
            }
        };

        if num_files > 1 {
            println!(
                "{}==> {} <==",
                if file_num > 0 { "\n" } else { "" },
                &filename
            );
        }

        if let Some(num_bytes) = config.bytes {
            let bytes: Result<Vec<u8>, _> = file.bytes().take(num_bytes).collect();
            print!("{}", String::from_utf8_lossy(&bytes?));
        } else {
            let mut line = String::new();
            for _ in 0..config.lines {
                let bytes = file.read_line(&mut line)?;
                if bytes == 0 {
                    break;
                }
                print!("{line}");
                line.clear();
            }
        }
    }
    Ok(())
}

fn open(filename: &str) -> RunResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?)))
    }
}

fn parse_positive_int(val: &str) -> RunResult<usize> {
    match val.parse() {
        Ok(n) if n > 0 => Ok(n),
        _ => Err(val.into())
    }    
}

#[test]
fn test_parse_positive_int() {
    let res = parse_positive_int("3");
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), 3);

    let res = parse_positive_int("foo");
    assert!(res.is_err());
    assert_eq!(res.unwrap_err().to_string(), "foo".to_string());

    let res = parse_positive_int("0");
    assert!(res.is_err());
    assert_eq!(res.unwrap_err().to_string(), "0".to_string());
}
