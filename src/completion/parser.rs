use clap::ArgMatches;

use crate::engine::{App, Core, Resolver};

/// The result of classifying a launch command's arguments.
///
/// Launch-style commands (`open`, `scratch`, `new`, `worktree`) declare a single
/// `args` positional and hand it to [`parse`], which classifies the tokens into a
/// strongly-typed target `T` (resolved via [`Resolver`]), an optional
/// application, and any number of `KEY=VALUE` environment overrides.
#[derive(Debug)]
pub struct ParsedArgs<T> {
    /// The resolved target the command should operate on (e.g. a [`crate::engine::Repo`],
    /// [`crate::engine::Scratchpad`] or [`crate::engine::Branch`]).
    pub target: T,
    /// The application named on the command line, if any. When `None` the command
    /// should fall back to the configured default (see [`ParsedArgs::launch_app`]).
    pub app: Option<App>,
    /// Literal environment overrides, in the order they were provided. The value
    /// keeps any additional `=` characters (e.g. `FOO=a=b` yields `("FOO", "a=b")`).
    pub env: Vec<(String, String)>,
}

impl<T> ParsedArgs<T> {
    /// Returns the application to launch — the one named on the command line, or
    /// the configured default when none was named — with the parsed environment
    /// overrides applied verbatim.
    pub fn launch_app(&self, core: &Core) -> Result<App, human_errors::Error> {
        let base: App = match &self.app {
            Some(app) => app.clone(),
            None => core.resolve(())?,
        };

        Ok(base.with_overrides(self.env.clone()))
    }
}

/// Classifies a launch command's `args` positional into a resolved target, an
/// optional application, and environment overrides.
///
/// `context` is the target implied by the current environment (e.g. the current
/// repository for `open`, the current scratchpad for `scratch`, or `None` when a
/// target must be given explicitly). It is both the fallback target and the
/// signal that a lone app-named token may be treated as an application.
///
/// A target is always required; the application is optional with a default. A
/// token that matches a configured app name is treated as the app only when a
/// target remains available (from another token or from `context`); otherwise it
/// is the target (target-priority). So `gt wt shell` resolves the branch `shell`,
/// `gt o shell` inside a repository launches the `shell` app in the current repo,
/// and `gt o shell` outside a repository opens the repo `shell`.
///
/// The target token is resolved through [`Resolver`], so the caller chooses what
/// `T` to resolve to simply by annotating the return type.
pub fn parse<T>(
    core: &Core,
    context: Option<&T>,
    matches: &ArgMatches,
) -> Result<ParsedArgs<T>, human_errors::Error>
where
    Core: for<'a> Resolver<&'a str, T>,
    T: Clone,
{
    let mut positionals: Vec<&str> = Vec::new();
    let mut env: Vec<(String, String)> = Vec::new();

    // A token is an environment override when it is shaped like `KEY=VALUE` and
    // `KEY` is a valid environment variable identifier; everything else is a
    // positional. The value retains any further `=` characters.
    if let Some(values) = matches.get_many::<String>("args") {
        for arg in values.map(|s| s.as_str()) {
            match arg.split_once('=') {
                Some((key, value)) if is_env_key(key) => {
                    env.push((key.to_string(), value.to_string()))
                }
                _ => positionals.push(arg),
            }
        }
    }

    let (app, target) = match positionals.as_slice() {
        [] => (None, resolve_context(context)?),
        // A single token becomes the app only when a target is still available
        // from the current context; otherwise target-priority makes it the target.
        [only] => match core.config().get_app(only) {
            Some(app) if context.is_some() => (Some(app.clone()), resolve_context(context)?),
            _ => (None, core.resolve(*only)?),
        },
        // With two tokens the one matching a configured app is the app and the
        // other is the target, preferring the first as the app when both match.
        [first, second] => {
            if let Some(app) = core.config().get_app(first) {
                (Some(app.clone()), core.resolve(*second)?)
            } else if let Some(app) = core.config().get_app(second) {
                (Some(app.clone()), core.resolve(*first)?)
            } else {
                return Err(human_errors::user(
                    format!(
                        "Could not find application with name '{first}'. Make sure that you are using an application which is present in your configuration file, or install it with 'git-tool config add apps/{first}'."
                    ),
                    &["Check your configuration file for available applications."],
                ));
            }
        }
        _ => {
            return Err(human_errors::user(
                "You provided too many arguments.",
                &[
                    "This command accepts at most an application and a target, along with any number of KEY=VALUE environment overrides.",
                ],
            ));
        }
    };

    Ok(ParsedArgs { target, app, env })
}

/// Resolves the fallback target from the current context, erroring when none is
/// available (a target is always required).
fn resolve_context<T: Clone>(context: Option<&T>) -> Result<T, human_errors::Error> {
    context.cloned().ok_or_else(|| {
        human_errors::user(
            "You did not specify a target to use.",
            &[
                "Provide a target (such as a repository, branch, or scratchpad name) when running this command.",
            ],
        )
    })
}

/// Returns `true` when `key` is a valid environment variable identifier
/// (`^[A-Za-z_][A-Za-z0-9_]*$`). An empty key is not valid, so tokens like
/// `=bar` are treated as positionals rather than environment overrides.
fn is_env_key(key: &str) -> bool {
    let mut chars = key.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' => {}
        _ => return false,
    }

    chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::Branch;

    fn core() -> Core {
        // The default configuration registers a `shell` application which we use
        // as the "known app" throughout these tests.
        Core::builder().with_default_config().build()
    }

    /// Builds an `ArgMatches` with the given tokens bound to the `args`
    /// positional, mirroring how the launch commands declare their arguments.
    fn matches(tokens: &[&str]) -> ArgMatches {
        clap::Command::new("test")
            .arg(clap::Arg::new("args").action(clap::ArgAction::Append))
            .get_matches_from(std::iter::once("test").chain(tokens.iter().copied()))
    }

    fn branch(name: &str) -> Branch {
        name.parse().unwrap()
    }

    // We resolve to `Branch` throughout: it resolves from a bare string without
    // touching the filesystem, so the tests focus purely on classification.

    #[test]
    fn normal_ordering() {
        let core = core();
        let parsed = parse::<Branch>(&core, None, &matches(&["shell", "some-branch"])).unwrap();
        assert_eq!(
            parsed.app.map(|a| a.get_name().to_string()),
            Some("shell".into())
        );
        assert_eq!(parsed.target.as_str(), "some-branch");
        assert!(parsed.env.is_empty());
    }

    #[test]
    fn odd_ordering() {
        let core = core();
        let parsed = parse::<Branch>(&core, None, &matches(&["some-branch", "shell"])).unwrap();
        assert_eq!(
            parsed.app.map(|a| a.get_name().to_string()),
            Some("shell".into())
        );
        assert_eq!(parsed.target.as_str(), "some-branch");
    }

    #[test]
    fn app_only_with_context() {
        let core = core();
        let parsed =
            parse::<Branch>(&core, Some(&branch("current")), &matches(&["shell"])).unwrap();
        assert_eq!(
            parsed.app.map(|a| a.get_name().to_string()),
            Some("shell".into())
        );
        assert_eq!(parsed.target.as_str(), "current");
    }

    #[test]
    fn target_only() {
        let core = core();
        let parsed = parse::<Branch>(&core, None, &matches(&["some-branch"])).unwrap();
        assert!(parsed.app.is_none());
        assert_eq!(parsed.target.as_str(), "some-branch");
    }

    #[test]
    fn app_named_token_without_context_is_target() {
        let core = core();
        // Without a context there is no fallback target, so target-priority makes
        // an app-named token the target.
        let parsed = parse::<Branch>(&core, None, &matches(&["shell"])).unwrap();
        assert!(parsed.app.is_none());
        assert_eq!(parsed.target.as_str(), "shell");
    }

    #[test]
    fn non_app_token_with_context_is_target() {
        let core = core();
        let parsed =
            parse::<Branch>(&core, Some(&branch("current")), &matches(&["other"])).unwrap();
        assert!(parsed.app.is_none());
        assert_eq!(parsed.target.as_str(), "other");
    }

    #[test]
    fn no_args_with_context_uses_context() {
        let core = core();
        let parsed = parse::<Branch>(&core, Some(&branch("current")), &matches(&[])).unwrap();
        assert!(parsed.app.is_none());
        assert_eq!(parsed.target.as_str(), "current");
    }

    #[test]
    fn no_args_without_context() {
        let core = core();
        assert!(
            parse::<Branch>(&core, None, &matches(&[]))
                .expect_err("a target is always required")
                .is(human_errors::Kind::User)
        );
    }

    #[test]
    fn unknown_app_with_two_tokens() {
        let core = core();
        assert!(
            parse::<Branch>(&core, None, &matches(&["unknown", "other"]))
                .expect_err("should error when neither token is an app")
                .is(human_errors::Kind::User)
        );
    }

    #[test]
    fn too_many_positionals_error() {
        let core = core();
        assert!(
            parse::<Branch>(&core, None, &matches(&["shell", "a", "b"]))
                .expect_err("more than two positionals is an error")
                .is(human_errors::Kind::User)
        );
    }

    #[test]
    fn env_simple() {
        let core = core();
        let parsed = parse::<Branch>(&core, None, &matches(&["some-branch", "FOO=bar"])).unwrap();
        assert_eq!(parsed.target.as_str(), "some-branch");
        assert_eq!(parsed.env, vec![("FOO".to_string(), "bar".to_string())]);
    }

    #[test]
    fn env_empty_value() {
        let core = core();
        let parsed = parse::<Branch>(&core, None, &matches(&["some-branch", "FOO="])).unwrap();
        assert_eq!(parsed.env, vec![("FOO".to_string(), String::new())]);
    }

    #[test]
    fn env_value_with_equals() {
        let core = core();
        let parsed = parse::<Branch>(&core, None, &matches(&["some-branch", "FOO=a=b"])).unwrap();
        assert_eq!(parsed.env, vec![("FOO".to_string(), "a=b".to_string())]);
    }

    #[test]
    fn invalid_env_key_is_target() {
        let core = core();
        let parsed = parse::<Branch>(&core, None, &matches(&["1FOO=bar"])).unwrap();
        assert!(parsed.app.is_none());
        assert_eq!(parsed.target.as_str(), "1FOO=bar");
        assert!(parsed.env.is_empty());
    }

    #[test]
    fn empty_env_key_is_target() {
        let core = core();
        let parsed = parse::<Branch>(&core, None, &matches(&["=bar"])).unwrap();
        assert_eq!(parsed.target.as_str(), "=bar");
        assert!(parsed.env.is_empty());
    }

    #[test]
    fn env_key_named_like_app_is_override() {
        let core = core();
        // Documented collision: `env=prod` is a valid KEY=VALUE override.
        let parsed =
            parse::<Branch>(&core, Some(&branch("current")), &matches(&["env=prod"])).unwrap();
        assert_eq!(parsed.target.as_str(), "current");
        assert_eq!(parsed.env, vec![("env".to_string(), "prod".to_string())]);
    }

    #[test]
    fn two_non_app_tokens_error() {
        let core = core();
        assert!(
            parse::<Branch>(&core, None, &matches(&["foo", "bar"]))
                .expect_err("neither token is an app")
                .is(human_errors::Kind::User)
        );
    }

    #[test]
    fn mixed_app_target_and_env() {
        let core = core();
        let parsed =
            parse::<Branch>(&core, None, &matches(&["shell", "some-branch", "FOO=bar"])).unwrap();
        assert_eq!(
            parsed.app.map(|a| a.get_name().to_string()),
            Some("shell".into())
        );
        assert_eq!(parsed.target.as_str(), "some-branch");
        assert_eq!(parsed.env, vec![("FOO".to_string(), "bar".to_string())]);
    }

    #[test]
    fn launch_app_defaults_and_applies_overrides() {
        let core = core();
        let parsed = parse::<Branch>(&core, None, &matches(&["some-branch", "FOO=bar"])).unwrap();

        // No app named, so we fall back to the default (`shell`) with overrides.
        let app = parsed.launch_app(&core).unwrap();
        assert_eq!(app.get_name(), "shell");
        assert!(
            app.get_overrides()
                .contains(&("FOO".to_string(), "bar".to_string()))
        );
    }
}
