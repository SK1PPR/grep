use crate::regex::elements::Matcher;

#[derive(Debug, Clone)]
pub struct State {
    pub id: usize,
    pub transitions: Vec<(Matcher, usize)>,
}

impl State {
    pub fn new(id: usize) -> Self {
        State {
            id,
            transitions: Vec::new(),
        }
    }

    pub fn add_transition(&mut self, c: Matcher, next_state_id: usize) {
        self.transitions.push((c, next_state_id));
    }

    pub fn shift_ids(&mut self, shift: usize) {
        self.id += shift;
        for (_, next_state_id) in &mut self.transitions {
            *next_state_id += shift;
        }
    }
}