use clap::{Arg, Command};
use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read};

#[derive(Debug)]
pub struct Config {
    files: Vec<String>,
    lines: isize,
    bytes: Option<isize>
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
        .map(String::clone)
        .collect();
    
    let lines = matches.get_one::<String>("lines")
        .map(String::as_str)
        .map(parse)
        .transpose()
        .map_err(|err|
            format!("illegal line count -- {err}")
        )?
        .unwrap();

    let bytes = matches.get_one::<String>("bytes")
        .map(String::as_str)
        .map(parse)
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
        let (mut file, size, line_count) = match open(&filename) {
            Ok((file, size, line_count)) => (file, size, line_count),
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
            let bytes: Result<Vec<u8>, _> =
                file.bytes().take(
                    if num_bytes < 0 {
                        size as isize + num_bytes
                    } else { num_bytes } as usize
                ).collect();
            print!("{}", String::from_utf8_lossy(&bytes?));
        } else {
            let mut line = String::new();
            let num_lines = if config.lines < 0 {
                line_count as isize + config.lines
            } else { config.lines };
            for _ in 0..num_lines {
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

fn open(filename: &str) -> RunResult<(Box<dyn BufRead>, usize, usize)> {
    if let "-" = filename {
        Ok((
        Box::new(BufReader::new(io::stdin())),
        0,
        1
    ))
    } else {
        let line_count = BufReader::new(File::open(filename)?).lines().count();
        let file = File::open(filename)?;
        let size = file.metadata()?.len();
        
        Ok((
            Box::new(BufReader::new(file)),
            size as usize,
            line_count
        ))
    }
}

fn parse(val: &str) -> RunResult<isize> {
    let map = "KMGTPEZ";
    let scale: isize;
    let mut len = val.len();

    match val.chars().last().unwrap() {
        'b' => {
            scale = 512;
            len -= 1;
        },
        'B' => {
            len -= 2;
            match val[0..=len].chars().last().unwrap() {
                'k' => scale = 1000,
                c => if let Some(n) = map.find(c) {
                    scale = 10_isize.pow(n as u32);
                } else { return Err(val.into()) }
            }
        }

        c => if let Some(n) = map.find(c) {
            scale = 1 << (10 * (n + 1));
            len -= 1;
        } else { scale = 1; }
    }

    match val[0..len].parse::<isize>() {
        Ok(n) => Ok(scale * n),
        _ => Err(val.into())
    }    
}

#[test]
fn test_parse() {
    let res = parse("3");
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), 3);

    let res = parse("3kB");
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), 3000);

    let res = parse("3G");
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), 3 << 30);

    let res = parse("foo");
    assert!(res.is_err());
    assert_eq!(res.unwrap_err().to_string(), "foo".to_string());
}
