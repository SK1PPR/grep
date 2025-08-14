use crate::regex::elements::{Matcher, State};

#[derive(Debug, Clone)]
pub struct Engine {
    pub states: Vec<State>,
    pub start_state: usize,
    pub end_state: usize,
}

impl Engine {
    pub fn new() -> Self {
        Engine {
            states: Vec::new(),
            start_state: 0,
            end_state: 0,
        }
    }

    pub fn add_states(&mut self, state: Vec<State>) {
        self.states.extend(state);
    }

    pub fn set_start_state(&mut self, state_id: usize) {
        self.start_state = state_id;
    }

    pub fn set_end_state(&mut self, state_id: usize) {
        self.end_state = state_id;
    }

    pub fn add_transition(&mut self, from_state: usize, matcher: Matcher, to_state: usize) {
        if let Some(state) = self.states.iter_mut().find(|s| s.id == from_state) {
            state.add_transition(matcher, to_state);
        }
    }

    pub fn compute(&self, input: &str) -> i32 {
        let mut stack: Vec<(usize, usize, Vec<usize>)> = vec![];
        stack.push((self.start_state, 0, Vec::new()));

        while stack.len() > 0 {
            let (current_state_id, input_index, memory) = stack.pop().unwrap();
            if current_state_id == self.end_state {
                return input_index as i32;
            }

            // Make sure we only make epsilon transitions if we are out of bounds
            if input_index >= input.chars().count() {
                if let Some(state) = self.states.iter().find(|s| s.id == current_state_id) {
                    for (_, next_state_id) in state
                        .transitions
                        .iter()
                        .rev()
                        .filter(|(m, _)| m.is_epsilon())
                    {
                        if memory.contains(&next_state_id) {
                            continue; // Avoid cycles
                        }
                        let mut memory = memory.clone();
                        memory.push(next_state_id.clone());
                        stack.push((next_state_id.clone(), input_index, memory.clone()));
                    }
                }
                continue;
            }

            let input_char = input[input
                .char_indices()
                .nth(input_index)
                .map(|(i, _)| i)
                .unwrap()..]
                .chars()
                .next()
                .unwrap();
            if let Some(state) = self.states.iter().find(|s| s.id == current_state_id) {
                for (matcher, next_state_id) in state
                    .transitions
                    .iter()
                    .rev()
                    .filter(|(m, _)| m.matches(input_char))
                {
                    if matcher.is_epsilon() {
                        if memory.contains(&next_state_id) {
                            continue; // Avoid cycles
                        }
                        let mut memory = memory.clone();
                        memory.push(next_state_id.clone());
                        stack.push((next_state_id.clone(), input_index, memory.clone()));
                    } else {
                        if input_index + 1 <= input.chars().count() {
                            stack.push((next_state_id.clone(), input_index + 1, Vec::new()));
                        }
                    }
                }
            }
        }

        return -1;
    }

    pub fn shift_ids(&mut self, shift: usize) {
        for state in &mut self.states {
            state.shift_ids(shift);
        }
        self.start_state += shift;
        self.end_state += shift;

        #[cfg(debug_assertions)]
        {
            println!("Shifted start state to {}", self.start_state);
            println!("Shifted end state to {}", self.end_state);
            println!("Shifted states are {:?}", self.states);
        }
    }
}
