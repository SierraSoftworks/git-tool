use crate::search::matches;
use std::sync::{Arc, Mutex};

pub struct Completer {
    filter: Arc<String>,
    output: Arc<Mutex<dyn std::io::Write + Send>>,
}

impl Completer {
    pub fn new(filter: &str) -> Self {
        Self::new_for(filter, Arc::new(Mutex::new(std::io::stdout())))
    }

    pub fn new_for(filter: &str, output: Arc<Mutex<dyn std::io::Write + Send>>) -> Self {
        Self {
            filter: Arc::new(filter.to_string()),
            output: output,
        }
    }

    pub fn offer(&self, completion: &str) {
        if matches(completion, &self.filter) {
            match self.output.lock().map(|mut out| {
                if has_whitespace(completion) {
                    writeln!(out, "'{}'", completion)
                } else {
                    writeln!(out, "{}", completion)
                }
            }) {
                _ => {}
            }
        }
    }

    pub fn offer_many<S, T>(&self, completions: S)
    where
        S: IntoIterator<Item = T>,
        T: Into<String>,
    {
        for item in completions {
            self.offer(&item.into());
        }
    }
}

fn has_whitespace(value: &str) -> bool {
    value.split_ascii_whitespace().skip(1).next().is_some()
}
