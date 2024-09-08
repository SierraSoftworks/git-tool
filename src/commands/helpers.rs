use crate::core::*;
use crate::errors;
use tracing_batteries::prelude::*;

pub enum LaunchTarget<'a> {
    AppAndTarget(&'a App, &'a str),
    App(&'a App),
    Target(&'a str),
    Err(errors::Error),
    None,
}

#[tracing::instrument(skip(core))]
pub fn get_launch_app<'a, S: AsRef<str> + std::fmt::Debug + std::fmt::Display + ?Sized>(
    core: &'a Core,
    first: Option<&'a S>,
    second: Option<&'a S>,
) -> LaunchTarget<'a> {
    match (first, second) {
        (Some(first), Some(second)) => {
            if let Some(app) = core.config().get_app(first.as_ref()) {
                LaunchTarget::AppAndTarget(app, second.as_ref())
            } else if let Some(app) = core.config().get_app(second.as_ref()) {
                LaunchTarget::AppAndTarget(app, first.as_ref())
            } else {
                LaunchTarget::Err(errors::user(
                    format!("Could not find application with name '{first}'.").as_str(),
                    format!("Make sure that you are using an application which is present in your configuration file, or install it with 'git-tool config add apps/{first}'.").as_str()))
            }
        }
        (Some(first), None) => {
            if let Some(app) = core.config().get_app(first.as_ref()) {
                LaunchTarget::App(app)
            } else {
                LaunchTarget::Target(first.as_ref())
            }
        }
        (None, Some(second)) => {
            if let Some(app) = core.config().get_app(second.as_ref()) {
                LaunchTarget::App(app)
            } else {
                LaunchTarget::Target(second.as_ref())
            }
        }
        (None, None) => LaunchTarget::None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normal_ordering() {
        let core = Core::builder().with_default_config().build();

        match get_launch_app(&core, Some("shell"), Some("gh:test/test")) {
            LaunchTarget::AppAndTarget(app, repo) => {
                assert_eq!(app.get_name(), "shell");
                assert_eq!(repo, "gh:test/test");
            }
            _ => panic!("Both the app and target should have been matched."),
        }
    }

    #[test]
    fn test_odd_ordering() {
        let core = Core::builder().with_default_config().build();

        match get_launch_app(&core, Some("gh:test/test"), Some("shell")) {
            LaunchTarget::AppAndTarget(app, repo) => {
                assert_eq!(app.get_name(), "shell");
                assert_eq!(repo, "gh:test/test");
            }
            _ => panic!("Both the app and target should have been matched."),
        }
    }

    #[test]
    fn test_app_only() {
        let core = Core::builder().with_default_config().build();

        match get_launch_app(&core, Some("shell"), None) {
            LaunchTarget::App(app) => {
                assert_eq!(app.get_name(), "shell");
            }
            _ => panic!("Only the app should have been matched."),
        }
    }

    #[test]
    fn test_target_only() {
        let core = Core::builder().with_default_config().build();

        match get_launch_app(&core, Some("gh:test/test"), None) {
            LaunchTarget::Target(repo) => {
                assert_eq!(repo, "gh:test/test");
            }
            _ => panic!("Only the target should have been matched."),
        }
    }

    #[test]
    fn test_unknown_app() {
        let core = Core::builder().with_default_config().build();

        match get_launch_app(&core, Some("unknown"), Some("gh:test/test")) {
            LaunchTarget::Err(e) => assert!(!e.is_system()),
            _ => panic!("Expected to receive an error."),
        }
    }

    #[test]
    fn test_no_args() {
        let core = Core::builder().with_default_config().build();

        match get_launch_app::<String>(&core, None, None) {
            LaunchTarget::None => {}
            _ => panic!("Expected to receive none"),
        }
    }
}
