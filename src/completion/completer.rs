use crate::{console::ConsoleProvider, search::matches};
use std::{fmt::Display, sync::Arc};

pub struct Completer {
    filter: Arc<String>,
    console: Arc<dyn ConsoleProvider + Send + Sync>,
}

impl Completer {
    pub fn new(core: &crate::core::Core, filter: &str) -> Self {
        Self::new_for(filter, core.console())
    }

    pub fn new_for(filter: &str, console: Arc<dyn ConsoleProvider + Send + Sync>) -> Self {
        Self {
            filter: Arc::new(filter.to_string()),
            console,
        }
    }

    pub fn offer(&self, completion: &str) {
        if !matches(completion, &self.filter) {
            return;
        }
        let mut out = self.console.output();
        if has_whitespace(completion) {
            writeln!(out, "'{completion}'").unwrap_or_default();
        } else {
            writeln!(out, "{completion}").unwrap_or_default();
        }
    }

    pub fn offer_many<S>(&self, completions: S)
    where
        S: IntoIterator,
        S::Item: AsRef<str> + Display + Clone,
    {
        let mut out = self.console.output();
        for completion in crate::search::best_matches(&self.filter, completions) {
            if has_whitespace(&completion) {
                writeln!(out, "'{}'", &completion).unwrap_or_default();
            } else {
                writeln!(out, "{}", &completion).unwrap_or_default();
            }
        }
    }
}

fn has_whitespace<T: AsRef<str>>(value: T) -> bool {
    value.as_ref().split_ascii_whitespace().nth(1).is_some()
}
