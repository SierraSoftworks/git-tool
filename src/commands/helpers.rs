use crate::core::*;
use crate::errors;

pub enum LaunchTarget<'a> {
    AppAndTarget(&'a App, &'a str),
    App(&'a App),
    Target(&'a str),
    Err(errors::Error),
    None,
}

pub fn get_launch_app<'a, C: Core>(
    core: &'a C,
    first: Option<&'a str>,
    second: Option<&'a str>,
) -> LaunchTarget<'a> {
    match (first, second) {
        (Some(first), Some(second)) => {
            if let Some(app) = core.config().get_app(first) {
                LaunchTarget::AppAndTarget(app, second)
            } else if let Some(app) = core.config().get_app(second) {
                LaunchTarget::AppAndTarget(app, first)
            } else {
                LaunchTarget::Err(errors::user(
                    format!("Could not find application with name '{}'.", first).as_str(),
                    format!("Make sure that you are using an application which is present in your configuration file, or install it with 'git-tool config add apps/{}'.", first).as_str()))
            }
        }
        (Some(first), None) => {
            if let Some(app) = core.config().get_app(first) {
                LaunchTarget::App(app)
            } else {
                LaunchTarget::Target(first)
            }
        }
        (None, Some(second)) => {
            if let Some(app) = core.config().get_app(second) {
                LaunchTarget::App(app)
            } else {
                LaunchTarget::Target(second)
            }
        }
        (None, None) => LaunchTarget::None,
    }
}
