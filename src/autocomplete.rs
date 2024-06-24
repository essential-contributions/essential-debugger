use std::collections::HashSet;

use inquire::Autocomplete;

#[derive(Default, Clone)]
pub struct Auto {
    history: std::sync::Arc<std::sync::Mutex<Inner>>,
}

#[derive(Default)]
struct Inner {
    history: Vec<String>,
    set: HashSet<String>,
}

impl Auto {
    pub fn update_history(&self, input: &str) {
        let mut h = self.history.lock().unwrap();
        if h.set.insert(input.to_string()) {
            h.history.push(input.to_string());
        }
    }
}

impl Autocomplete for Auto {
    fn get_suggestions(&mut self, input: &str) -> Result<Vec<String>, inquire::CustomUserError> {
        Ok(self
            .history
            .lock()
            .unwrap()
            .history
            .iter()
            .rev()
            .filter(|s| s.starts_with(input))
            .cloned()
            .collect())
    }

    fn get_completion(
        &mut self,
        _input: &str,
        highlighted_suggestion: Option<String>,
    ) -> Result<inquire::autocompletion::Replacement, inquire::CustomUserError> {
        Ok(highlighted_suggestion)
    }
}
