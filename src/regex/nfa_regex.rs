use std::vec;

use crate::regex::elements::{Matcher, State};
use crate::regex::engine::Engine;
use crate::regex::parser::Token;

#[allow(dead_code)]
pub struct RegexNFA {
    pub engine: Engine,
    pattern: String, 
    starts_with: bool,
    ends_with: bool,
}

enum Quantifier {
    Star,
    Question,
    Plus,
}

impl RegexNFA {
    pub fn new(pattern: String) -> Self {
        let tokens = crate::regex::parser::postfix_generator(&pattern);
        let engine = create_engine(&tokens);
        let starts_with = matches!(tokens.first(), Some(Token::StartRef));
        let ends_with = matches!(tokens.last(), Some(Token::EndRef));
        RegexNFA {
            engine,
            pattern,
            starts_with,
            ends_with,
        }
    }

    pub fn matches(&self, input: &str) -> bool {
        if input.is_empty() {
            return self.engine.compute(input) != -1;
        }

        if self.starts_with {
            let index = self.engine.compute(input);
            if index >= 0 {
                if self.ends_with {
                    if index == input.len() as i32 {
                        return true; // Matches the entire input
                    }
                    return false;
                }
                return true; // Matches from the start
            }
            return false;
        }

        // Slice input and keep checking until found
        for i in 0..input.len() {
            let slice = input.chars().skip(i).take(input.len() - i).collect::<String>();
            let index = self.engine.compute(&slice);
            if index >= 0 {
                if self.ends_with {
                    if index as usize + i == input.len() {
                        return true; // Matches the entire input
                    }
                    return false;
                }
                return true; // Found a match
            }
        }

        return false;
    }
}

fn create_engine(tokens: &Vec<Token>) -> Engine {

    let mut engine_stack: Vec<Engine> = vec![];

    let mut iter = tokens.iter().peekable();
    while let Some(token) = iter.next() {
        match token {
            Token::Literal(c) => {
                let nfa = literal_nfa(c.clone());
                engine_stack.push(nfa);
            }
            Token::ComplexLiteral(s) => {
                let nfa = comple_nfa(&s);
                engine_stack.push(nfa);
            }
            Token::Star => {
                if let Some(next_token) = iter.peek() {
                    if next_token == &&Token::Question {
                        iter.next();
                        let engine = engine_stack.pop().expect("Expected engine for star");
                        let nfa = special_nfa_quantifier(engine, true, Quantifier::Star);
                        engine_stack.push(nfa);
                        continue;
                    }
                }

                let engine = engine_stack.pop().expect("Expected engine for star");
                let nfa = special_nfa_quantifier(engine, false, Quantifier::Star);
                engine_stack.push(nfa);
            }
            Token::Question => {
                if let Some(next_token) = iter.peek() {
                    if next_token == &&Token::Question {
                        iter.next();
                        let engine = engine_stack.pop().expect("Expected engine for question");
                        let nfa = special_nfa_quantifier(engine, true, Quantifier::Plus);
                        engine_stack.push(nfa);
                        continue;
                    }
                }

                let engine = engine_stack.pop().expect("Expected engine for question");
                let nfa = special_nfa_quantifier(engine, false, Quantifier::Question);
                engine_stack.push(nfa);
            }
            Token::Plus => {
                if let Some(next_token) = iter.peek() {
                    if next_token == &&Token::Question {
                        iter.next();
                        let engine = engine_stack.pop().expect("Expected engine for plus");
                        let nfa = special_nfa_quantifier(engine, true, Quantifier::Plus);
                        engine_stack.push(nfa);
                        continue;
                    }
                }

                let engine = engine_stack.pop().expect("Expected engine for plus");
                let nfa = special_nfa_quantifier(engine, false, Quantifier::Plus);
                engine_stack.push(nfa);
            }
            Token::Or => {
                let right = engine_stack.pop().expect("Expected right engine for union");
                let left = engine_stack.pop().expect("Expected left engine for union");
                let nfa = union_nfa(left, right);
                engine_stack.push(nfa);
            }
            Token::Concat => {
                let right = engine_stack
                    .pop()
                    .expect("Expected right engine for concat");
                let left = engine_stack.pop().expect("Expected left engine for concat");
                let nfa = concat_nfa(left, right);
                engine_stack.push(nfa);
            }
            Token::StartRef | Token::EndRef => {}
            _ => {
                panic!("Unexpected token: {:?}", token);
            }
        }
    }

    assert_eq!(
        engine_stack.len(),
        1,
        "Expected exactly one engine in stack after processing tokens"
    );
    engine_stack.pop().expect("Expected final engine")
}

fn one_step_nfa(matcher: Matcher) -> Engine {
    let mut engine = Engine::new();
    let mut start_state = State::new(0);
    let end_state = State::new(1);
    start_state.add_transition(matcher, 1);
    engine.add_states(vec![start_state, end_state]);
    engine.set_start_state(0);
    engine.set_end_state(1);
    engine
}

fn literal_nfa(c: char) -> Engine {
    one_step_nfa(Matcher::create_simple_matcher(&c))
}

fn comple_nfa(input: &str) -> Engine {
    one_step_nfa(Matcher::create_complex_matcher(input))
}

fn union_nfa(left: Engine, mut right: Engine) -> Engine {
    let mut engine = Engine::new();
    let start_state_id = left.states.len() + right.states.len();
    let end_state_id = start_state_id + 1;

    engine.add_states(left.states.clone());
    right.shift_ids(left.states.len());
    engine.add_states(right.states);

    engine.set_start_state(start_state_id);
    engine.set_end_state(end_state_id);

    engine.add_states(vec![State::new(start_state_id), State::new(end_state_id)]);

    // Add epsilon transitions from the start state to both left and right engines
    engine.add_transition(start_state_id, Matcher::Epsilon, left.start_state);
    engine.add_transition(start_state_id, Matcher::Epsilon, right.start_state);

    // Add epsilon transitions from both left and right engines to the end state
    engine.add_transition(left.end_state, Matcher::Epsilon, end_state_id);
    engine.add_transition(right.end_state, Matcher::Epsilon, end_state_id);

    #[cfg(debug_assertions)]
    {
        println!(
            "Created concat NFA with start state {} and end state {}",
            start_state_id, end_state_id
        );

        println!("Final states: {:?}", engine.states);
    }

    engine
}

fn concat_nfa(left: Engine, mut right: Engine) -> Engine {
    let mut engine = Engine::new();
    let start_state_id = left.states.len() + right.states.len();
    let end_state_id = start_state_id + 1;

    engine.add_states(left.states.clone());
    right.shift_ids(left.states.len());
    engine.add_states(right.states);

    engine.set_start_state(start_state_id);
    engine.set_end_state(end_state_id);

    engine.add_states(vec![State::new(start_state_id), State::new(end_state_id)]);

    // Add epsilon transition from the end of left to the start of right
    engine.add_transition(left.end_state, Matcher::Epsilon, right.start_state);

    // Add transitions from the start state to the left engine
    engine.add_transition(start_state_id, Matcher::Epsilon, left.start_state);

    // Add transitions from the end of right to the end state
    engine.add_transition(right.end_state, Matcher::Epsilon, end_state_id);

    #[cfg(debug_assertions)]
    {
        println!(
            "Created concat NFA with start state {} and end state {}",
            start_state_id, end_state_id
        );

        println!("Final states: {:?}", engine.states);
    }

    engine
}

fn special_nfa_quantifier(engine: Engine, lazy: bool, quantifier: Quantifier) -> Engine {
    let mut new_engine = Engine::new();
    let start_state_id = engine.states.len();
    let end_state_id = start_state_id + 1;

    new_engine.add_states(engine.states.clone());
    new_engine.set_start_state(start_state_id);
    new_engine.set_end_state(end_state_id);

    new_engine.add_states(vec![State::new(start_state_id), State::new(end_state_id)]);

    // Order of epsilon transitions depends on wether the quantifier is lazy or not
    match quantifier {
        Quantifier::Star => {
            if lazy {
                new_engine.add_transition(start_state_id, Matcher::Epsilon, engine.start_state);
                new_engine.add_transition(start_state_id, Matcher::Epsilon, end_state_id);
                new_engine.add_transition(engine.end_state, Matcher::Epsilon, start_state_id);
                new_engine.add_transition(engine.end_state, Matcher::Epsilon, end_state_id);
            } else {
                new_engine.add_transition(start_state_id, Matcher::Epsilon, end_state_id);
                new_engine.add_transition(start_state_id, Matcher::Epsilon, engine.start_state);
                new_engine.add_transition(engine.end_state, Matcher::Epsilon, end_state_id);
                new_engine.add_transition(engine.end_state, Matcher::Epsilon, start_state_id);
            }
        }
        Quantifier::Question => {
            if lazy {
                new_engine.add_transition(start_state_id, Matcher::Epsilon, end_state_id);
                new_engine.add_transition(start_state_id, Matcher::Epsilon, engine.start_state);
            } else {
                new_engine.add_transition(start_state_id, Matcher::Epsilon, engine.start_state);
                new_engine.add_transition(start_state_id, Matcher::Epsilon, end_state_id);
            }
            new_engine.add_transition(engine.end_state, Matcher::Epsilon, end_state_id);
        }
        Quantifier::Plus => {
            if lazy {
                new_engine.add_transition(engine.end_state, Matcher::Epsilon, end_state_id);
                new_engine.add_transition(engine.end_state, Matcher::Epsilon, start_state_id);
            } else {
                new_engine.add_transition(engine.end_state, Matcher::Epsilon, start_state_id);
                new_engine.add_transition(engine.end_state, Matcher::Epsilon, end_state_id);
            }
            new_engine.add_transition(start_state_id, Matcher::Epsilon, engine.start_state);
        }
    }

    #[cfg(debug_assertions)]
    {
        println!(
            "Created special NFA with start state {} and end state {}",
            start_state_id, end_state_id
        );

        println!("Final states: {:?}", new_engine.states);
    }

    new_engine
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_character_match() {
        let pattern = "a".to_string();
        let regex_nfa = RegexNFA::new(pattern);
        assert_eq!(regex_nfa.pattern, "a");
        assert!(regex_nfa.matches("aaab"));
        assert!(regex_nfa.matches("ab"));
        assert!(regex_nfa.matches("bba"));
    }

    #[test]
    fn test_concat_match() {
        let pattern = "ab".to_string();
        let regex_nfa = RegexNFA::new(pattern);
        assert_eq!(regex_nfa.pattern, "ab");
        assert!(!regex_nfa.matches("aaac"));
        assert!(regex_nfa.matches("cab"));
        assert!(regex_nfa.matches("ab"));
        assert!(!regex_nfa.matches("bba"));
    }

    #[test]
    fn test_union_match() {
        let pattern = "a|b".to_string();
        let regex_nfa = RegexNFA::new(pattern);
        assert_eq!(regex_nfa.pattern, "a|b");
        assert!(regex_nfa.matches("a"));
        assert!(regex_nfa.matches("b"));
        assert!(!regex_nfa.matches("c"));
        assert!(regex_nfa.matches("cccccab"));
    }

    // Test quantifiers
    #[test]
    fn test_plus_match() {
        let pattern = "a+".to_string();
        let regex_nfa = RegexNFA::new(pattern);
        assert_eq!(regex_nfa.pattern, "a+");
        assert!(regex_nfa.matches("aaaa"));
        assert!(!regex_nfa.matches(""));
        assert!(!regex_nfa.matches("b"));
        assert!(regex_nfa.matches("bbaaa"));
    }

    #[test]
    fn test_question_match() {
        let pattern = "a?".to_string();
        let regex_nfa = RegexNFA::new(pattern);
        assert_eq!(regex_nfa.pattern, "a?");
        assert!(regex_nfa.matches("a"));
        assert!(regex_nfa.matches(""));
        assert!(regex_nfa.matches("b"));
        assert!(regex_nfa.matches("bba"));
    }

    #[test]
    fn test_star_match() {
        let pattern = "a*".to_string();
        let regex_nfa = RegexNFA::new(pattern);
        assert_eq!(regex_nfa.pattern, "a*");
        assert!(regex_nfa.matches("aaaa"));
        assert!(regex_nfa.matches(""));
        assert!(regex_nfa.matches("b"));
        assert!(regex_nfa.matches("bbaaa"));
    }

    // Start ref and end ref tests
    #[test]
    fn test_start_ref_match() {
        let pattern = "^a".to_string();
        let regex_nfa = RegexNFA::new(pattern);
        assert_eq!(regex_nfa.pattern, "^a");
        assert!(regex_nfa.matches("abc"));
        assert!(!regex_nfa.matches("bca"));
        assert!(!regex_nfa.matches("cba"));
    }

    #[test]
    fn test_end_ref_match() {
        let pattern = "a$".to_string();
        let regex_nfa = RegexNFA::new(pattern);
        assert_eq!(regex_nfa.pattern, "a$");
        assert!(regex_nfa.matches("ba"));
        assert!(!regex_nfa.matches("ab"));
        assert!(regex_nfa.matches("cba"));
    }

    // TODO: Test lazy quantifiers
}
