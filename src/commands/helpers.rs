use crate::engine::*;
use crate::errors;
use tracing_batteries::prelude::*;

#[derive(Debug)]
pub enum LaunchTarget<'a> {
    AppAndTarget(&'a App, Identifier),
    App(&'a App),
    Target(Identifier),
    None,
}

#[tracing::instrument(skip(core))]
pub fn get_launch_app<'a, S: AsRef<str> + std::fmt::Debug + std::fmt::Display + ?Sized>(
    core: &'a Core,
    first: Option<&'a S>,
    second: Option<&'a S>,
) -> Result<LaunchTarget<'a>, Error> {
    match (first, second) {
        (Some(first), Some(second)) => {
            if let Some(app) = core.config().get_app(first.as_ref()) {
                Ok(LaunchTarget::AppAndTarget(app, second.as_ref().parse()?))
            } else if let Some(app) = core.config().get_app(second.as_ref()) {
                Ok(LaunchTarget::AppAndTarget(app, first.as_ref().parse()?))
            } else {
                Err(errors::user(
                    format!("Could not find application with name '{first}'.").as_str(),
                    format!("Make sure that you are using an application which is present in your configuration file, or install it with 'git-tool config add apps/{first}'.").as_str()))
            }
        }
        (Some(field), None) | (None, Some(field)) => {
            if let Some(app) = core.config().get_app(field.as_ref()) {
                Ok(LaunchTarget::App(app))
            } else {
                Ok(LaunchTarget::Target(field.as_ref().parse()?))
            }
        }
        (None, None) => Ok(LaunchTarget::None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normal_ordering() {
        let core = Core::builder().with_default_config().build();

        match get_launch_app(&core, Some("shell"), Some("gh:test/test")).unwrap() {
            LaunchTarget::AppAndTarget(app, repo) => {
                assert_eq!(app.get_name(), "shell");
                assert_eq!(repo.to_string(), "gh:test/test");
            }
            _ => panic!("Both the app and target should have been matched."),
        }
    }

    #[test]
    fn test_odd_ordering() {
        let core = Core::builder().with_default_config().build();

        match get_launch_app(&core, Some("gh:test/test"), Some("shell")).unwrap() {
            LaunchTarget::AppAndTarget(app, repo) => {
                assert_eq!(app.get_name(), "shell");
                assert_eq!(repo.to_string(), "gh:test/test");
            }
            _ => panic!("Both the app and target should have been matched."),
        }
    }

    #[test]
    fn test_app_only() {
        let core = Core::builder().with_default_config().build();

        match get_launch_app(&core, Some("shell"), None).unwrap() {
            LaunchTarget::App(app) => {
                assert_eq!(app.get_name(), "shell");
            }
            _ => panic!("Only the app should have been matched."),
        }
    }

    #[test]
    fn test_target_only() {
        let core = Core::builder().with_default_config().build();

        match get_launch_app(&core, Some("gh:test/test"), None).unwrap() {
            LaunchTarget::Target(repo) => {
                assert_eq!(repo.to_string(), "gh:test/test");
            }
            _ => panic!("Only the target should have been matched."),
        }
    }

    #[test]
    fn test_unknown_app() {
        let core = Core::builder().with_default_config().build();

        assert!(get_launch_app(&core, Some("unknown"), Some("gh:test/test"))
            .expect_err("receive an error.")
            .is_user())
    }

    #[test]
    fn test_no_args() {
        let core = Core::builder().with_default_config().build();

        match get_launch_app::<String>(&core, None, None).expect("no issues") {
            LaunchTarget::None => {}
            _ => panic!("Expected to receive none"),
        }
    }
}
