#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Plus,
    Star,
    Question,
    Literal(char),
    EndRef,
    StartRef,
    ComplexLiteral(String),
    LBracket,
    RBracket,
    Concat,
    Or,
    None,
}

fn parse(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();
    let mut current_token = Token::None;

    while let Some(c) = chars.next() {
        match c {
            '+' => {
                if let Token::ComplexLiteral(ref mut s) = current_token {
                    s.push('+');
                } else {
                    tokens.push(Token::Plus);
                }
            }
            '*' => {
                if let Token::ComplexLiteral(ref mut s) = current_token {
                    s.push('*');
                } else {
                    tokens.push(Token::Star);
                }
            }
            '?' => {
                if let Token::ComplexLiteral(ref mut s) = current_token {
                    s.push('?');
                } else {
                    tokens.push(Token::Question);
                }
            }
            '$' => {
                if let Token::ComplexLiteral(ref mut s) = current_token {
                    s.push('$');
                } else {
                    tokens.push(Token::EndRef);
                }
            }
            '^' => {
                if let Token::ComplexLiteral(ref mut s) = current_token {
                    s.push('^');
                } else {
                    tokens.push(Token::StartRef);
                }
            }
            '|' => {
                if let Token::ComplexLiteral(ref mut s) = current_token {
                    s.push('|');
                } else {
                    tokens.push(Token::Or);
                }
            }
            '[' => {
                current_token = Token::ComplexLiteral(String::from('['));
            }
            ']' => {
                if let Token::ComplexLiteral(ref mut s) = current_token {
                    s.push(']');
                    tokens.push(current_token);
                    current_token = Token::None;
                } else {
                    panic!("Unmatched closing bracket in regex");
                }
            }
            '(' => {
                if let Token::ComplexLiteral(ref mut s) = current_token {
                    s.push('(');
                } else {
                    tokens.push(Token::LBracket);
                }
            }
            ')' => {
                if let Token::ComplexLiteral(ref mut s) = current_token {
                    s.push(')');
                } else {
                    tokens.push(Token::RBracket);
                }
            }
            '\\' => {
                if let Some(next_char) = chars.next() {
                    match next_char {
                        'd' => tokens.push(Token::ComplexLiteral("\\d".to_string())), // Placeholder for digit
                        'w' => tokens.push(Token::ComplexLiteral("\\w".to_string())), // Placeholder for word character
                        's' => tokens.push(Token::ComplexLiteral("\\s".to_string())), // Placeholder for whitespace
                        _ => tokens.push(Token::Literal(next_char)),
                        // TODO: Handle back references and other escape sequences
                    }
                } else {
                    panic!("Invalid escape sequence in regex");
                }
            }
            '.' => {
                if let Token::ComplexLiteral(ref mut s) = current_token {
                    s.push('.');
                } else {
                    tokens.push(Token::ComplexLiteral(".".to_string())); // Placeholder for dot
                }
            }
            _ => {
                if current_token == Token::None {
                    tokens.push(Token::Literal(c));
                } else {
                    if let Token::ComplexLiteral(ref mut s) = current_token {
                        s.push(c);
                    } else {
                        panic!("Unexpected character after complex literal start");
                    }
                }
            }
        }

        if current_token != Token::None && chars.peek().is_none() {
            panic!("Invalid regex");
        }
    }

    let mut final_tokens = Vec::new();
    let mut iter = tokens.into_iter().peekable();

    while let Some(token) = iter.next() {
        final_tokens.push(token.clone());

        if let Some(next) = iter.peek() {
            if needs_concat(&token, next) {
                final_tokens.push(Token::Concat);
            }
        }
    }

    final_tokens
}

fn needs_concat(prev: &Token, next: &Token) -> bool {
    matches!(
        prev,
        Token::Literal(_)
            | Token::ComplexLiteral(_)
            | Token::RBracket
            | Token::Star
            | Token::Plus
            | Token::Question
    ) && matches!(
        next,
        Token::Literal(_) | Token::ComplexLiteral(_) | Token::LBracket
    )
}

pub fn postfix_generator(input: &str) -> Vec<Token> {
    let tokens = parse(input);

    // Remove startPrefix and endPrefix tokens
    let parsed_tokens: Vec<Token> = tokens
        .into_iter()
        .filter(|token| *token != Token::StartRef && *token != Token::EndRef)
        .collect();

    let mut output = Vec::new();
    let mut stack = Vec::new();

    for token in parsed_tokens {
        match token {
            Token::Literal(_) | Token::ComplexLiteral(_) => {
                output.push(token);
            }
            Token::Plus | Token::Star | Token::Question => {
                stack.push(token);
            }
            Token::Concat => {
                while let Some(top) = stack.last() {
                    if matches!(top, Token::Plus | Token::Star | Token::Question) {
                        output.push(stack.pop().unwrap());
                    } else {
                        break;
                    }
                }
                stack.push(token);
            }
            Token::Or => {
                while let Some(top) = stack.last() {
                    if *top != Token::LBracket && *top != Token::RBracket {
                        output.push(stack.pop().unwrap());
                    } else {
                        break;
                    }
                }
                stack.push(token);
            }
            Token::LBracket => stack.push(token),
            Token::RBracket => {
                while let Some(top) = stack.last() {
                    if *top != Token::LBracket {
                        output.push(stack.pop().unwrap());
                    } else {
                        stack.pop(); // Pop the left bracket
                        break;
                    }
                }
            }
            _ => {}
        }
    }

    while let Some(top) = stack.pop() {
        output.push(top);
    }

    // Add back the start and end references if they were present
    let tokens = parse(input);
    if tokens.contains(&Token::StartRef) {
        output.insert(0, Token::StartRef);
    }
    if tokens.contains(&Token::EndRef) {
        output.push(Token::EndRef);
    }

    return output;
}

#[cfg(test)]
mod tests {
    use crate::regex::parser::{postfix_generator, Token};

    fn to_string(tokens: Vec<Token>) -> String {
    tokens
        .into_iter()
        .map(|token| match token {
            Token::Plus => "+".to_string(),
            Token::Star => "*".to_string(),
            Token::Question => "?".to_string(),
            Token::Literal(c) => c.to_string(),
            Token::EndRef => "$".to_string(),
            Token::StartRef => "^".to_string(),
            Token::ComplexLiteral(s) => s,
            Token::LBracket => "(".to_string(),
            Token::RBracket => ")".to_string(),
            Token::Concat => ".".to_string(), // Concat is implicit
            Token::Or => "|".to_string(),
            _ => "".to_string(), // Handle other tokens if needed
        })
        .collect()
}

fn to_postfix(input: &str) -> String {
    let tokens = postfix_generator(input);
    to_string(tokens)
}


    #[test]
    fn test_single_literal() {
        assert_eq!(to_postfix("a"), "a");
    }

    #[test]
    fn test_simple_concat() {
        assert_eq!(to_postfix("ab"), "ab.");
    }

    #[test]
    fn test_union() {
        assert_eq!(to_postfix("a|b"), "ab|");
    }

    #[test]
    fn test_kleene_star() {
        assert_eq!(to_postfix("a*"), "a*");
    }

    #[test]
    fn test_plus() {
        assert_eq!(to_postfix("a+"), "a+");
    }

    #[test]
    fn test_question() {
        assert_eq!(to_postfix("a?"), "a?");
    }

    #[test]
    fn test_concat_and_star() {
        assert_eq!(to_postfix("ab*"), "ab*.");
    }

    #[test]
    fn test_star_and_concat() {
        assert_eq!(to_postfix("a*b"), "a*b.");
    }

    #[test]
    fn test_union_and_concat() {
        assert_eq!(to_postfix("ab|c"), "ab.c|");
    }

    #[test]
    fn test_parens_simple() {
        assert_eq!(to_postfix("(ab)c"), "ab.c.");
    }

    #[test]
    fn test_parens_and_union() {
        assert_eq!(to_postfix("(a|b)c"), "ab|c.");
    }

    #[test]
    fn test_nested_parens() {
        assert_eq!(to_postfix("a(b(c|d))"), "abcd|..");
    }

    #[test]
    fn test_union_with_kleene() {
        assert_eq!(to_postfix("a*|b"), "a*b|");
    }

    #[test]
    fn test_complex() {
        assert_eq!(to_postfix("a(b|c)*d"), "abc|*d..");
    }

    #[test]
    fn test_question_and_union() {
        assert_eq!(to_postfix("a?|b"), "a?b|");
    }

    #[test]
    fn test_plus_and_question() {
        assert_eq!(to_postfix("a+?"), "a?+");
    }

    #[test]
    fn test_range_charclass() {
        assert_eq!(to_postfix("[abc]d"), "[abc]d.");
    }

    #[test]
    fn test_negated_charclass() {
        assert_eq!(to_postfix("[^abc]x"), "[^abc]x.");
    }
}
