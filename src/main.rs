use std::env;
use std::io;
use std::process;

fn match_pattern(input_line: &str, pattern: &str) -> bool {
    if pattern == r"\d" {
        return input_line.chars().any(char::is_numeric);
    } else if pattern == r"\w" {
        return input_line.chars().any(char::is_alphanumeric);
    } else if pattern.starts_with('[') && pattern.ends_with(']') {
        if pattern.chars().nth(1).unwrap() == '^' {
            return pattern.chars().any(|c| c != '[' && c != '^' && c != ']' && !input_line.contains(c))
        }
        return pattern.chars().any(|c| input_line.contains(c));
    } else if pattern.chars().count() == 1 {
        return input_line.contains(pattern);
    } else {
        panic!("Unhandled pattern: {}", pattern)
    }
}

// Usage: echo <input_text> | your_program.sh -E <pattern>
fn main() {
    if env::args().nth(1).unwrap() != "-E" {
        println!("Expected first argument to be '-E'");
        process::exit(1);
    }

    let pattern = env::args().nth(2).unwrap();
    let mut input_line = String::new();

    io::stdin().read_line(&mut input_line).unwrap();

    if match_pattern(&input_line, &pattern) {
        process::exit(0)
    } else {
        process::exit(1)
    }
}
