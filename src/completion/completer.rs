use crate::search::matches;
use std::{
    fmt::Display,
    sync::{Arc, Mutex},
};

pub struct Completer {
    filter: Arc<String>,
    output: Arc<Mutex<dyn std::io::Write + Send>>,
}

impl Completer {
    pub fn new(core: &crate::core::Core, filter: &str) -> Self {
        let output = core.output();
        Self::new_for(filter, Arc::new(Mutex::new(output)))
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

    pub fn offer_many<S>(&self, completions: S)
    where
        S: IntoIterator,
        S::Item: AsRef<str> + Display + Clone,
    {
        self.output
            .lock()
            .map(|mut out| {
                for completion in crate::search::best_matches(&self.filter, completions) {
                    if has_whitespace(&completion) {
                        writeln!(out, "'{}'", &completion).unwrap_or_default();
                    } else {
                        writeln!(out, "{}", &completion).unwrap_or_default();
                    }
                }
            })
            .unwrap_or_default();
    }
}

fn has_whitespace<T: AsRef<str>>(value: T) -> bool {
    value
        .as_ref()
        .split_ascii_whitespace()
        .skip(1)
        .next()
        .is_some()
}
