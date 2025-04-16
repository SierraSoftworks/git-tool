use crate::engine::Core;
use crate::{console::ConsoleProvider, search::matches};
use itertools::Itertools;
use std::{fmt::Display, sync::Arc};

pub struct Completer {
    filter: Arc<String>,
    console: Arc<dyn ConsoleProvider + Send + Sync>,
}

impl Completer {
    pub fn new(core: &Core, filter: &str) -> Self {
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
                    .unique()
                    .sorted(),
            );
            self.offer_many(
                repos
                    .iter()
                    .map(|r| format!("{}:{}/", &r.service, r.namespace))
                    .unique()
                    .sorted(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::console;
    use crate::engine::Core;
    use crate::test::get_dev_dir;

    #[test]
    fn test_offer() {
        let console = console::mock();
        let completer = Completer::new_for("test", console.clone());

        completer.offer("test");
        completer.offer("test test");
        completer.offer("not a match");

        assert_eq!(console.to_string(), "test\n'test test'\n");
    }

    #[test]
    fn test_offer_many() {
        let console = console::mock();
        let completer = Completer::new_for("test", console.clone());

        completer.offer_many(["test", "test test", "not a match"]);

        assert_eq!(console.to_string(), "test\n'test test'\n");
    }

    #[test]
    fn test_offer_aliases() {
        let console = console::mock();
        let mut config = crate::engine::Config::for_dev_directory(&get_dev_dir());
        config.add_alias("test1", "gh:test/example1");
        config.add_alias("test2", "gh:test/example2");

        let core = Core::builder()
            .with_config(config)
            .with_console(console.clone())
            .build();

        let completer = Completer::new_for("test", console.clone());
        completer.offer_aliases(&core);

        assert_eq!(
            console
                .to_string()
                .split_terminator('\n')
                .sorted()
                .collect::<Vec<_>>(),
            &["test1", "test2"]
        );
    }

    #[test]
    fn test_offer_apps() {
        let console = console::mock();
        let core = Core::builder()
            .with_config_for_dev_directory(get_dev_dir())
            .with_console(console.clone())
            .build();

        let completer = Completer::new_for("sh", console.clone());
        completer.offer_apps(&core);

        assert_eq!(
            console
                .to_string()
                .split_terminator('\n')
                .sorted()
                .collect::<Vec<_>>(),
            &["shell"]
        );
    }

    #[test]
    fn test_offer_namespaces() {
        let console = console::mock();
        let core = Core::builder()
            .with_config_for_dev_directory(get_dev_dir())
            .with_console(console.clone())
            .build();

        let completer = Completer::new_for("gh:", console.clone());
        completer.offer_namespaces(&core);

        assert_eq!(
            console
                .to_string()
                .split_terminator('\n')
                .sorted()
                .collect::<Vec<_>>(),
            &["gh:sierrasoftworks/", "gh:spartan563/"]
        );
    }

    #[test]
    fn test_offer_repos() {
        let console = console::mock();
        let core = Core::builder()
            .with_config_for_dev_directory(get_dev_dir())
            .with_console(console.clone())
            .build();

        let completer = Completer::new_for("gh:test1", console.clone());
        completer.offer_repos(&core);

        assert_eq!(
            console
                .to_string()
                .split_terminator('\n')
                .sorted()
                .collect::<Vec<_>>(),
            &[
                "gh:sierrasoftworks/test1",
                "gh:sierrasoftworks/test12",
                "gh:spartan563/test1"
            ]
        );
    }
}
