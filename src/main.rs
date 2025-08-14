use std::env;
use std::fs::{read_dir, File};
use std::io;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process;

mod regex;

use regex::RegexNFA;

fn match_pattern(input_line: &str, pattern: &str) -> bool {
    let regex_nfa = RegexNFA::new(pattern.to_string());
    regex_nfa.matches(input_line)
}

fn process_file(file_path: &str, pattern: &str, multiple: bool) -> io::Result<()> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut found_match = false;

    for (_, line) in reader.lines().enumerate() {
        let line = line?;
        if match_pattern(&line, pattern) {
            found_match = true;
            if multiple {
                println!("{}:{}", file_path, line);
            } else {
                println!("{}", line);
            }
        }
    }

    if !found_match {
        return Err(io::Error::new(io::ErrorKind::Other, "No matches found"));
    }
    Ok(())
}

fn process_directory_recursive(dir_path: &str, pattern: &str) -> io::Result<()> {
    let path = Path::new(dir_path);
    if !path.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Path is not a directory",
        ));
    }

    let mut found_match = false;

    for entry in read_dir(path)? {
        let entry = entry?;
        let entry_path = entry.path();

        if entry_path.is_file() {
            // Process file
            if let Ok(file) = File::open(&entry_path) {
                let reader = BufReader::new(file);

                for (_, line) in reader.lines().enumerate() {
                    if let Ok(line) = line {
                        if match_pattern(&line, pattern) {
                            println!("{}:{}", entry_path.display(), line);
                            found_match = true;
                        }
                    }
                }
            }
        } else if entry_path.is_dir() {
            // Recursively process subdirectory
            if let Some(dir_name) = entry_path.file_name() {
                if let Some(dir_name_str) = dir_name.to_str() {
                    // Skip hidden directories (starting with .)
                    if !dir_name_str.starts_with('.') {
                        match process_directory_recursive(entry_path.to_str().unwrap(), pattern) {
                            Ok(_) => found_match = true,
                            Err(_) => {}
                        }
                    }
                }
            }
        }
    }

    if !found_match {
        return Err(io::Error::new(io::ErrorKind::Other, "No matches found"));
    }
    Ok(())
}

fn process_stdin(pattern: &str) -> io::Result<()> {
    let stdin = io::stdin();
    let reader = stdin.lock();
    let mut found_match = false;

    for (_, line) in reader.lines().enumerate() {
        let line = line?;
        if match_pattern(&line, pattern) {
            found_match = true;
            println!("{}", line);
        }
    }

    if !found_match {
        return Err(io::Error::new(io::ErrorKind::Other, "No matches found"));
    }
    Ok(())
}

// Usage:
// echo <input_text> | myprogram -E <pattern>
// myprogram -E <pattern> <filepath1> [filepath2] [filepath3] ...
// myprogram -r -E <pattern> <directory1> [directory2] [directory3] ...
fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        println!("Usage: myprogram -E <pattern> [filepath1] [filepath2] ...");
        println!("       myprogram -r -E <pattern> <directory1> [directory2] ...");
        println!("  If no filepath is provided, reads from stdin");
        process::exit(1);
    }

    let mut recursive = false;
    let mut pattern_index = 0;
    let mut path_start_index = 0;

    // Parse arguments
    for (i, arg) in args.iter().enumerate() {
        match arg.as_str() {
            "-r" => recursive = true,
            "-E" => {
                pattern_index = i + 1;
                path_start_index = i + 2;
            }
            _ => {}
        }
    }

    if pattern_index == 0 {
        println!("Expected '-E' flag");
        process::exit(1);
    }

    if pattern_index >= args.len() {
        println!("Missing pattern after -E");
        process::exit(1);
    }

    let pattern = &args[pattern_index];

    // Check if paths are provided
    if path_start_index < args.len() {
        let paths = &args[path_start_index..];
        let mut found_match_anywhere = false;
        let mut errors = Vec::new();

        for path in paths {
            let path_result = if recursive {
                // Recursive directory search
                process_directory_recursive(path, pattern)
            } else {
                // Single file search
                process_file(path, pattern, paths.len() > 1)
            };

            match path_result {
                Ok(_) => {
                    found_match_anywhere = true;
                }
                Err(e) => {
                    let error_msg = format!("Error processing '{}': {}", path, e);
                    errors.push(error_msg);
                }
            }
        }

        // Exit with appropriate code
        if found_match_anywhere {
            process::exit(0);
        } else {
            if !found_match_anywhere && !errors.is_empty() {
                for error in errors {
                    eprintln!("{}", error);
                }
            }
            // No matches found in any file
            process::exit(1);
        }
    } else {
        // No path provided, read from stdin
        match process_stdin(pattern) {
            Ok(_) => process::exit(0),
            Err(e) => {
                eprintln!("Error reading from stdin: {}", e);
                process::exit(1);
            }
        }
    }
}
