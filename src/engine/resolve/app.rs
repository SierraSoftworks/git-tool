use super::Resolver;
use crate::engine::{App, Core};

/// Resolves the default application (the first one in the configuration file).
impl Resolver<(), App> for Core {
    fn resolve(&self, _source: ()) -> Result<App, human_errors::Error> {
        self.config().get_default_app().cloned().ok_or_else(|| {
            human_errors::user(
                "No default application available.",
                &["Make sure that you add an app to your config file using 'git-tool config add apps/bash' or similar."],
            )
        })
    }
}

/// Resolves an application by its configured name.
impl Resolver<&str, App> for Core {
    fn resolve(&self, source: &str) -> Result<App, human_errors::Error> {
        self.config().get_app(source).cloned().ok_or_else(|| {
            human_errors::user(
                format!(
                    "Could not find application with name '{source}'. Make sure that you are using an application which is present in your configuration file, or install it with 'git-tool config add apps/{source}'."
                ),
                &["Check your configuration file for available applications."],
            )
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn core() -> Core {
        Core::builder().with_default_config().build()
    }

    #[test]
    fn resolves_an_app_by_name() {
        let core = core();
        let app: App = core.resolve("shell").unwrap();
        assert_eq!(app.get_name(), "shell");

        assert!(Resolver::<&str, App>::resolve(&core, "not-an-app").is_err());
    }

    #[test]
    fn resolves_the_default_app() {
        let core = core();
        let app: App = core.resolve(()).unwrap();
        assert_eq!(app.get_name(), "shell");
    }
}
