use core::panic;

#[derive(Debug, Clone, PartialEq)]
pub enum Matcher {
    Range(Vec<char>, bool), // Range of characters, e.g., 'a' to 'z', and if is negated
    Epsilon,
}

impl Matcher {
    pub fn is_epsilon(&self) -> bool {
        matches!(self, Matcher::Epsilon)
    }

    pub fn matches(&self, c: char) -> bool {
        match self {
            Matcher::Range(chars, negated) => {
                let contains = chars.contains(&c);
                if *negated { !contains } else { contains }
            }
            Matcher::Epsilon => true, // Epsilon matches all charcters
        }
    }

    fn create_alphanumeric() -> Matcher {
        Matcher::Range(
            ('a'..='z')
                .chain('A'..='Z')
                .chain('0'..='9')
                .chain(std::iter::once('_'))
                .collect(),
            false,
        )
    }

    fn create_digit() -> Matcher {
        Matcher::Range(('0'..='9').collect(), false)
    }

    fn create_blank(negated: bool) -> Matcher {
        Matcher::Range(Vec::new(), negated)
    }

    fn append_literal(mut matcher: Matcher, c: char) -> Matcher {
        if let Matcher::Range(ref mut chars, negated) = matcher {
            chars.push(c);
            Matcher::Range(chars.clone(), negated)
        } else {
            matcher
        }
    }

    fn create_dot() -> Matcher {
        // Matches any character except \n and \r
        Matcher::Range(
            ('\u{0000}'..='\u{10FFFF}')
                .filter(|&c| c != '\n' && c != '\r')
                .collect(),
            false,
        )
    }

    pub fn create_complex_matcher(input: &str) -> Matcher {
        match input.len() {
            1 => match input.chars().next().unwrap() {
                '.' => Matcher::create_dot(),
                'd' => Matcher::create_digit(),
                'w' => Matcher::create_alphanumeric(),
                _ => panic!("Unknown complex token: {}", input),
            },
            2 => {
                panic!("Complex tokens with length 2 are not supported: {}", input);
            }
            _ => {
                // All regex of the form [..]
                // Remove the first and last characters

                let inner = &input[1..input.len() - 1];
                if inner.is_empty() {
                    panic!("Empty character class is not allowed");
                }

                let negated = inner.starts_with('^');
                let inner = if negated { &inner[1..] } else { inner };
                if inner.is_empty() {
                    panic!("Empty character class is not allowed");
                }
                let mut chars = Vec::new();

                // Split the '-' into seperated ranges
                let range_ends = inner.split('-').collect::<Vec<&str>>();
                let mut prev_char = '\0';
                for range_end in range_ends {
                    if range_end.is_empty() {
                        // the regex was of the form [^-]
                        chars.push('-');
                        prev_char = '-';
                    } else {
                        if prev_char != '\0' {
                            // We have a range
                            let start = prev_char;
                            let end = range_end.chars().next().unwrap();
                            if start > end {
                                panic!("Invalid range in character class: {}-{}", start, end);
                            }
                            chars.extend(start..=end);
                        } 
                        // Add the current characters
                        for c in range_end.chars() {
                            chars.push(c);
                            prev_char = c;
                        }
                    }
                }

                // Remove duplicates from chars
                chars.sort();
                chars.dedup();

                return Matcher::Range(
                    chars,
                    negated,
                );
            }
        }
    }

    pub fn create_simple_matcher(input: &char) -> Matcher {
        Matcher::append_literal(Matcher::create_blank(false), *input)
    }
}