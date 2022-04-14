use super::async_trait;
use super::*;
use super::{core::Target, tasks, tasks::Task, Command};
use clap::{Arg, ArgMatches};

pub struct ScratchCommand {}

impl Command for ScratchCommand {
    fn name(&self) -> String {
        String::from("scratch")
    }

    fn app<'a>(&self) -> clap::Command<'a> {
        clap::Command::new(self.name().as_str())
            .version("1.0")
            .visible_alias("s")
            .about("opens a scratchpad using an application defined in your config")
            .arg(
                Arg::new("app")
                    .help("The name of the application to launch.")
                    .index(1),
            )
            .arg(
                Arg::new("scratchpad")
                    .help("The name of the scratchpad to open.")
                    .index(2),
            )
    }
}

#[async_trait]
impl CommandRunnable for ScratchCommand {
    #[tracing::instrument(name = "gt scratch", err, skip(self, core, matches))]
    async fn run(&self, core: &Core, matches: &ArgMatches) -> Result<i32, errors::Error> {
        let (app, scratchpad) = match helpers::get_launch_app(
            core,
            matches.value_of("app"),
            matches.value_of("scratchpad"),
        ) {
            helpers::LaunchTarget::AppAndTarget(app, target) => {
                (app, core.resolver().get_scratchpad(target)?)
            }
            helpers::LaunchTarget::App(app) => (app, core.resolver().get_current_scratchpad()?),
            helpers::LaunchTarget::Target(target) => {
                let app = core.config().get_default_app().ok_or(errors::user(
                    "No default application available.",
                    "Make sure that you add an app to your config file using 'git-tool config add apps/bash' or similar."))?;

                (app, core.resolver().get_scratchpad(target)?)
            }
            helpers::LaunchTarget::Err(err) => return Err(err),
            helpers::LaunchTarget::None => {
                let app = core.config().get_default_app().ok_or(errors::user(
                    "No default application available.",
                    "Make sure that you add an app to your config file using 'git-tool config add apps/bash' or similar."))?;

                (app, core.resolver().get_current_scratchpad()?)
            }
        };

        if !scratchpad.exists() {
            let task = tasks::NewFolder {};
            task.apply_scratchpad(core, &scratchpad).await?;
        }

        let status = core.launcher().run(app, &scratchpad).await?;
        return Ok(status);
    }

    #[tracing::instrument(
        name = "gt complete -- gt scratch",
        skip(self, core, completer, _matches)
    )]
    async fn complete(&self, core: &Core, completer: &Completer, _matches: &ArgMatches) {
        completer.offer_many(core.config().get_apps().map(|a| a.get_name()));

        match core.resolver().get_scratchpads() {
            Ok(pads) => {
                completer.offer_many(pads.iter().map(|p| p.get_name()));
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::*;
    use mocktopus::mocking::*;

    #[tokio::test]
    async fn run_no_args() {
        let cmd = ScratchCommand {};

        let args = cmd.app().get_matches_from(vec!["scratch"]);

        let temp = tempfile::tempdir().unwrap();
        let cfg = Config::from_str(&format!(
            "
directory: {}
scratchpads: {}

apps:
  - name: test-app
    command: test
    args:
        - '{{ .Target.Name }}'
",
            temp.path().display(),
            temp.path().join("scratch").display()
        ))
        .unwrap();

        let temp_path = temp.path().to_owned();
        Resolver::get_current_scratchpad.mock_safe(move |_| {
            MockResult::Return(Ok(Scratchpad::new(
                "2020w01",
                temp_path.join("scratch").join("2020w01").into(),
            )))
        });

        Launcher::run.mock_safe(move |_, app, target| {
            assert_eq!(
                app.get_name(),
                "test-app",
                "it should launch the correct app"
            );

            assert_eq!(
                target.get_path(),
                temp.path().join("scratch").join("2020w01"),
                "the app should be launched in the correct directory"
            );

            MockResult::Return(Box::pin(async move { Ok(5) }))
        });

        let core = Core::builder().with_config(&cfg).build();

        match cmd.run(&core, &args).await {
            Ok(status) => {
                assert_eq!(status, 5, "it should forward the status code from the app");
            }
            Err(err) => panic!("{}", err.message()),
        }
    }

    #[tokio::test]
    async fn run_only_app() {
        let cmd = ScratchCommand {};

        let args = cmd.app().get_matches_from(vec!["scratch", "test-app"]);

        let temp = tempfile::tempdir().unwrap();
        let cfg = Config::from_str(&format!(
            "
directory: {}
scratchpads: {}

apps:
  - name: test-app
    command: test
    args:
        - '{{ .Target.Name }}'
",
            temp.path().display(),
            temp.path().join("scratch").display()
        ))
        .unwrap();

        let temp_path = temp.path().to_owned();
        Resolver::get_current_scratchpad.mock_safe(move |_| {
            MockResult::Return(Ok(Scratchpad::new(
                "2020w01",
                temp_path.join("scratch").join("2020w01").into(),
            )))
        });

        Launcher::run.mock_safe(move |_, app, target| {
            assert_eq!(
                app.get_name(),
                "test-app",
                "it should launch the correct app"
            );
            assert_eq!(
                target.get_path(),
                temp.path().join("scratch").join("2020w01"),
                "the app should be launched in the correct directory"
            );

            MockResult::Return(Box::pin(async move { Ok(0) }))
        });

        let core = Core::builder().with_config(&cfg).build();

        match cmd.run(&core, &args).await {
            Ok(_) => {}
            Err(err) => panic!("{}", err.message()),
        }
    }

    #[tokio::test]
    async fn run_only_scratchpad() {
        let cmd = ScratchCommand {};

        let args = cmd.app().get_matches_from(vec!["scratch", "2020w07"]);

        let temp = tempfile::tempdir().unwrap();
        let cfg = Config::from_str(&format!(
            "
directory: {}
scratchpads: {}

apps:
  - name: test-app
    command: test
    args:
        - '{{ .Target.Name }}'
        ",
            temp.path().display(),
            temp.path().join("scratch").display()
        ))
        .unwrap();

        let temp_path = temp.path().to_owned();
        Resolver::get_scratchpad.mock_safe(move |_, name| {
            assert_eq!(
                name, "2020w07",
                "it should attempt to resolve the correct scratchpad name"
            );

            MockResult::Return(Ok(Scratchpad::new(
                "2020w07",
                temp_path.join("scratch").join("2020w07").into(),
            )))
        });

        Launcher::run.mock_safe(move |_, app, target| {
            assert_eq!(
                app.get_name(),
                "test-app",
                "it should launch the correct app"
            );
            assert_eq!(
                target.get_path(),
                temp.path().join("scratch").join("2020w07"),
                "the app should be launched in the correct directory"
            );

            MockResult::Return(Box::pin(async move { Ok(0) }))
        });

        let core = Core::builder().with_config(&cfg).build();

        match cmd.run(&core, &args).await {
            Ok(_) => {}
            Err(err) => panic!("{}", err.message()),
        }
    }

    #[tokio::test]
    async fn run_app_and_scratchpad() {
        let cmd = ScratchCommand {};

        let args = cmd
            .app()
            .get_matches_from(vec!["scratch", "test-app", "2020w07"]);

        let temp = tempfile::tempdir().unwrap();
        let cfg = Config::from_str(&format!(
            "
directory: {}
scratchpads: {}

apps:
  - name: test-app
    command: test
    args:
        - '{{ .Target.Name }}'
",
            temp.path().display(),
            temp.path().join("scratch").display()
        ))
        .unwrap();

        let temp_path = temp.path().to_owned();
        Resolver::get_scratchpad.mock_safe(move |_, name| {
            assert_eq!(
                name, "2020w07",
                "it should attempt to resolve the correct scratchpad name"
            );

            MockResult::Return(Ok(Scratchpad::new(
                "2020w07",
                temp_path.join("scratch").join("2020w07").into(),
            )))
        });

        Launcher::run.mock_safe(move |_, app, target| {
            assert_eq!(
                app.get_name(),
                "test-app",
                "it should launch the correct app"
            );
            assert_eq!(
                target.get_path(),
                temp.path().join("scratch").join("2020w07"),
                "the app should be launched in the correct directory"
            );

            MockResult::Return(Box::pin(async move { Ok(0) }))
        });

        let core = Core::builder().with_config(&cfg).build();

        match cmd.run(&core, &args).await {
            Ok(_) => {}
            Err(err) => panic!("{}", err.message()),
        }
    }

    #[tokio::test]
    async fn run_unknown_app() {
        let cmd = ScratchCommand {};

        let args = cmd
            .app()
            .get_matches_from(vec!["scratch", "unknown-app", "2020w07"]);

        let temp = tempfile::tempdir().unwrap();
        let cfg = Config::from_str(&format!(
            "
directory: {}
scratchpads: {}

apps:
  - name: test-app
    command: test
    args:
        - '{{ .Target.Name }}'
",
            temp.path().display(),
            temp.path().join("scratch").display()
        ))
        .unwrap();

        Resolver::get_scratchpad.mock_safe(move |_, name| {
            assert_eq!(
                name, "2020w07",
                "it should attempt to resolve the correct scratchpad name"
            );

            MockResult::Return(Ok(Scratchpad::new(
                "2020w07",
                temp.path().join("scratch").join("2020w07").into(),
            )))
        });

        Launcher::run.mock_safe(|_, _app, _target| {
            panic!("It should not launch an app.");
        });

        let core = Core::builder().with_config(&cfg).build();

        match cmd.run(&core, &args).await {
            Ok(_) => {}
            Err(_) => {}
        }
    }

    #[tokio::test]
    async fn run_new_scratchpad() {
        let cmd = ScratchCommand {};

        let args = cmd.app().get_matches_from(vec!["scratch", "2020w07"]);

        let temp = tempfile::tempdir().unwrap();
        let cfg = Config::from_str(&format!(
            "
directory: {}
scratchpads: {}

apps:
  - name: test-app
    command: test
    args:
        - '{{ .Target.Name }}'
",
            temp.path().display(),
            temp.path().join("scratch").display()
        ))
        .unwrap();

        let temp_path = temp.path().to_owned();
        Resolver::get_scratchpad.mock_safe(move |_, name| {
            assert_eq!(
                name, "2020w07",
                "it should attempt to resolve the correct scratchpad name"
            );

            MockResult::Return(Ok(Scratchpad::new(
                "2020w07",
                temp_path.join("scratch").join("2020w07").into(),
            )))
        });

        Launcher::run.mock_safe(move |_, app, target| {
            assert_eq!(
                app.get_name(),
                "test-app",
                "it should launch the correct app"
            );
            assert_eq!(
                target.get_path(),
                temp.path().join("scratch").join("2020w07"),
                "the app should be launched in the correct directory"
            );

            assert!(
                target.get_path().exists(),
                "the target directory should be created"
            );

            MockResult::Return(Box::pin(async move { Ok(0) }))
        });

        let core = Core::builder().with_config(&cfg).build();

        match cmd.run(&core, &args).await {
            Ok(_) => {}
            Err(err) => panic!("{}", err.message()),
        }
    }
}
