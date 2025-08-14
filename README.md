# Rust Grep Clone

A command-line implementation of the `grep` utility written in Rust. This project demonstrates building a text search tool with regular expression support.

## Features

- Text pattern matching using regular expressions
- File and directory searching
- Command-line interface similar to Unix grep
- Custom regex engine built from scratch (no external regex libraries)

## Implementation Details

The regex engine is built entirely from scratch — no external regex crates are used. It uses the Shunting Yard algorithm to parse into postfix notation and a Thompson-style NFA for matching. This favors clarity and correctness over micro-optimizations. It’s not the most optimized engine, but it’s practical and works well for typical use cases.

## Building

Build the project:

```sh
cargo build --release
```

Run the built binary directly:

```sh
./target/release/grep-clone -E "pattern" [paths...]
```

Optionally install it to your Cargo bin so it’s available as a command (recommended):

```sh
cargo install --path .
```

If installed, you can run it as `myprogram` by creating an alias or symlink, or by renaming the package/binary. Examples below use `myprogram`; if not aliased, replace `myprogram` with `./target/release/grep-clone`.

## Running

Synopsis:

```sh
myprogram [-r] -E "pattern" [path1] [path2] ...
```

Notes:
- If no paths are provided, the program reads from stdin.
- With `-r`, each provided path is treated as a directory to search recursively (files inside are searched).
- You can pass multiple files and/or directories.

Examples:

- Read from stdin (pipe):
```sh
echo "asdfasdf" | myprogram -E "pattern"
```

- Search a single file:
```sh
myprogram -E "pattern" path/to/file.txt
```

- Search multiple files:
```sh
myprogram -E "pattern" file1.txt file2.log file3.md
```

- Search directories recursively:
```sh
myprogram -r -E "pattern" path/to/dir1 path/to/dir2
```

- Mixed files and directories:
```sh
myprogram -r -E "pattern" file1.txt path/to/dir file2.txt
```

Output format:
- From stdin or a single file: matching lines are printed.
- From multiple files or recursive search: `path:line` is printed for each match.

## Project Structure

- `src/main.rs` - Main entry point and CLI handling
- `src/regex/` - Regular expression engine implementation
  - `parser.rs` - Regex parsing logic
  - `engine.rs` - Regex matching engine
  - `nfa_regex.rs` - NFA-based regex implementation
  - `elements/` - Regex element definitions and matchers

## Further improvements
- Add `Backreferences`
- Add test cases to check lazy matching quantifiers
- Convert NFA to DFA to improve performance (at the cost of memory)
- Simplify the NFAs to reduce states

## Dependencies

This project uses Rust's standard library and has no external dependencies.

## License

This project is open source and available under the MIT License.
