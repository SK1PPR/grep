# Rust Grep Clone

A command-line implementation of the `grep` utility written in Rust. This project demonstrates building a text search tool with regular expression support.

## Features

- Text pattern matching using regular expressions
- File content searching
- Command-line interface similar to Unix grep
- Efficient regex parsing and matching engine

## Implementation Details

The regex engine is made entirely by myself using shunting yard and Thompson NFA. This provides a robust and efficient way to parse and match regular expressions.

## Building

To build the project:

```sh
cargo build --release
```

## Running

To run the grep clone:

```sh
cargo run -- "pattern" "filename"
```

Or after building:

```sh
./target/release/grep-starter-rust "pattern" "filename"
```

## Project Structure

- `src/main.rs` - Main entry point and CLI handling
- `src/regex/` - Regular expression engine implementation
  - `parser.rs` - Regex parsing logic
  - `engine.rs` - Regex matching engine
  - `nfa_regex.rs` - NFA-based regex implementation
  - `elements/` - Regex element definitions and matchers

## Dependencies

This project uses Rust's standard library and has no external dependencies.

## License

This project is open source and available under the MIT License.
