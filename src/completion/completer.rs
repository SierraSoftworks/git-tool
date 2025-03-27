use crate::core::Core;
use crate::{console::ConsoleProvider, search::matches};
use itertools::Itertools;
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

    pub fn offer<S: AsRef<str>>(&self, completion: S) {
        let completion = completion.as_ref();
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

    pub fn offer_aliases(&self, core: &Core) {
        self.offer_many(core.config().get_aliases().map(|(a, _)| a));
    }

    pub fn offer_apps(&self, core: &Core) {
        self.offer_many(core.config().get_apps().map(|a| a.get_name()));
    }

    pub fn offer_namespaces(&self, core: &Core) {
        let default_svc = core
            .config()
            .get_default_service()
            .map(|s| s.name.clone())
            .unwrap_or_default();

        if let Ok(repos) = core.resolver().get_repos() {
            self.offer_many(
                repos
                    .iter()
                    .filter(|r| r.service == default_svc)
                    .map(|r| format!("{}/", r.namespace))
                    .unique(),
            );
            self.offer_many(
                repos
                    .iter()
                    .map(|r| format!("{}:{}/", &r.service, r.namespace))
                    .unique(),
            );
        }
    }

    pub fn offer_repos(&self, core: &Core) {
        let default_svc = core
            .config()
            .get_default_service()
            .map(|s| s.name.clone())
            .unwrap_or_default();

        if let Ok(repos) = core.resolver().get_repos() {
            self.offer_many(
                repos
                    .iter()
                    .filter(|r| r.service == default_svc)
                    .map(|r| r.get_full_name()),
            );
            self.offer_many(
                repos
                    .iter()
                    .map(|r| format!("{}:{}", &r.service, r.get_full_name())),
            );
        }
    }
}

fn has_whitespace<T: AsRef<str>>(value: T) -> bool {
    value.as_ref().split_ascii_whitespace().nth(1).is_some()
}
