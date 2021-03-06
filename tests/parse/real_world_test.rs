extern crate walkdir;

use self::walkdir::WalkDir;
use javaparser::{parse, tokenize};
use std::io::Write;
use std::path::Path;
use std::time::{Duration, Instant};
use std::{fs, io};

#[test]
#[ignore]
fn test() {
    let mut slowest_duration = Duration::from_millis(0);
    let mut slowest_path = String::new();
    let mut failures = vec![];
    let mut successes = vec![];
    for entry in WalkDir::new("/home/tanin/projects/jdk12u-390566f1850a/src")
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.is_dir() {
            continue;
        }

        let filename = path.file_name().unwrap().to_str().unwrap();
        if !filename.ends_with(".java") {
            continue;
        }

        if filename == "package-info.java" || filename == "module-info.java" {
            continue;
        }

        print!("Parsed {} ", path.to_str().unwrap());
        let _ = io::stdout().flush();

        match parse_file(path) {
            Ok(duration) => {
                if duration > slowest_duration {
                    slowest_duration = duration;
                    slowest_path = String::from(path.to_str().unwrap());
                }
                successes.push(String::from(path.to_str().unwrap()));
                println!("succeeded ({:?})", duration);
            }
            Err(_) => {
                failures.push(String::from(path.to_str().unwrap()));
                println!("failed")
            }
        };
    }

    println!(
        "Succeeded: {} files, failed: {} files",
        successes.len(),
        failures.len()
    );
    println!("Slowest file: {} ({:?})", slowest_path, slowest_duration);
}

fn parse_file(path: &Path) -> Result<Duration, ()> {
    let start = Instant::now();

    let content = fs::read_to_string(path).unwrap();
    let result = parse::apply(&content, path.to_str().unwrap());

    match result {
        Ok(_) => Ok(start.elapsed()),
        Err(_) => Err(()),
    }
}
