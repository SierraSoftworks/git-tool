use chrono::prelude::*;

use super::{ResolveMany, Resolver, TrueResolver};
use crate::engine::{Core, Scratchpad};
use crate::fs::get_child_directories;
use tracing_batteries::prelude::*;

/// Resolves the current week's scratchpad.
impl Resolver<(), Scratchpad> for Core {
    fn resolve(&self, source: ()) -> Result<Scratchpad, human_errors::Error> {
        self.resolve_with_events(source)
    }
}

/// Resolves a scratchpad by name, supporting the `^N`/`~N` week-offset syntax
/// (e.g. `^1` is last week's scratchpad).
impl Resolver<&str, Scratchpad> for Core {
    fn resolve(&self, source: &str) -> Result<Scratchpad, human_errors::Error> {
        self.resolve_with_events(source)
    }
}

/// Enumerates the scratchpads which currently exist on disk.
impl ResolveMany<(), Scratchpad> for Core {
    fn resolve_many(&self, source: ()) -> Result<Vec<Scratchpad>, human_errors::Error> {
        self.resolve_many_with_events(source)
    }
}

impl Resolver<(), Scratchpad> for TrueResolver {
    #[tracing::instrument(err, skip(self, _source))]
    fn resolve(&self, _source: ()) -> Result<Scratchpad, human_errors::Error> {
        let time = Local::now();

        self.resolve(time.format("%Yw%V").to_string().as_str())
    }
}

impl Resolver<&str, Scratchpad> for TrueResolver {
    #[tracing::instrument(err, skip(self, name))]
    fn resolve(&self, name: &str) -> Result<Scratchpad, human_errors::Error> {
        if name.starts_with('^') || name.starts_with('~') {
            let delta = name[1..].parse::<u64>().map_err(|err| {
                human_errors::user(
                    format!(
                        "Could not parse the offset expression '{}' into a valid week offset: {}.",
                        &name, err,
                    ),
                    &["Please provide a valid number of weeks to go back in time."],
                )
            })?;

            let time = Local::now() - chrono::Duration::days(delta as i64 * 7);

            return self.resolve(time.format("%Yw%V").to_string().as_str());
        }

        Ok(Scratchpad::new(
            name,
            self.config.get_scratch_directory().join(name),
        ))
    }
}

impl ResolveMany<(), Scratchpad> for TrueResolver {
    #[tracing::instrument(err, skip(self, _source))]
    fn resolve_many(&self, _source: ()) -> Result<Vec<Scratchpad>, human_errors::Error> {
        Ok(get_child_directories(&self.config.get_scratch_directory())?
            .filter_map(|p| {
                p.file_name()
                    .and_then(|f| f.to_str())
                    .map(|name| Scratchpad::new(name, p.to_path_buf()))
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::{Config, Target};
    use crate::test::get_dev_dir;
    use std::sync::Arc;

    fn resolver() -> TrueResolver {
        TrueResolver::new(Arc::new(Config::for_dev_directory(&get_dev_dir())))
    }

    fn core() -> Core {
        Core::builder()
            .with_config(Config::for_dev_directory(&get_dev_dir()))
            .build()
    }

    #[test]
    fn resolves_a_scratchpad_by_name() {
        let core = core();
        let scratchpad: Scratchpad = core.resolve("2019w15").unwrap();
        assert_eq!(
            scratchpad.get_path(),
            get_dev_dir().join("scratch").join("2019w15")
        );
    }

    #[test]
    fn resolves_a_scratchpad_which_does_not_exist_yet() {
        let resolver = resolver();
        let scratchpad: Scratchpad = resolver.resolve("2019w10").unwrap();
        assert_eq!(
            scratchpad.get_path(),
            get_dev_dir().join("scratch").join("2019w10")
        );
    }

    #[test]
    fn resolves_week_offsets() {
        let resolver = resolver();

        let scratchpad: Scratchpad = resolver.resolve("^0").unwrap();
        assert_eq!(
            scratchpad.get_path(),
            get_dev_dir()
                .join("scratch")
                .join(Local::now().format("%Yw%V").to_string())
        );

        let scratchpad: Scratchpad = resolver.resolve("^1").unwrap();
        assert_eq!(
            scratchpad.get_path(),
            get_dev_dir().join("scratch").join(
                (Local::now() - chrono::Duration::days(7))
                    .format("%Yw%V")
                    .to_string()
            )
        );

        let scratchpad: Scratchpad = resolver.resolve("^5").unwrap();
        assert_eq!(
            scratchpad.get_path(),
            get_dev_dir().join("scratch").join(
                (Local::now() - chrono::Duration::days(7 * 5))
                    .format("%Yw%V")
                    .to_string()
            )
        );

        assert!(Resolver::<&str, Scratchpad>::resolve(&resolver, "^not-a-number").is_err());
        assert!(Resolver::<&str, Scratchpad>::resolve(&resolver, "^-1").is_err());
    }

    #[test]
    fn resolves_the_current_weeks_scratchpad() {
        let core = core();
        let name = Local::now().format("%Yw%V").to_string();

        let scratchpad: Scratchpad = core.resolve(()).unwrap();
        assert_eq!(scratchpad.get_name(), name);
        assert_eq!(
            scratchpad.get_path(),
            get_dev_dir().join("scratch").join(name)
        );
    }

    #[test]
    fn enumerates_existing_scratchpads() {
        let core = core();
        let scratchpads: Vec<Scratchpad> = core.resolve_many(()).unwrap();
        assert_eq!(scratchpads.len(), 3);
        assert!(scratchpads.iter().any(|s| s.get_name() == "2019w15"));
        assert!(scratchpads.iter().any(|s| s.get_name() == "2019w16"));
        assert!(scratchpads.iter().any(|s| s.get_name() == "2019w27"));
    }
}
