use super::async_trait;
use super::*;
use super::{core::Target, tasks::Task};
use clap::Arg;
use tracing_batteries::prelude::*;

pub struct ScratchCommand;
crate::command!(ScratchCommand);

#[async_trait]
impl CommandRunnable for ScratchCommand {
    fn name(&self) -> String {
        String::from("scratch")
    }

    fn app(&self) -> clap::Command {
        clap::Command::new(self.name())
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

    #[tracing::instrument(name = "gt scratch", err, skip(self, core, matches))]
    async fn run(&self, core: &Core, matches: &ArgMatches) -> Result<i32, errors::Error> {
        let (app, scratchpad) = match helpers::get_launch_app(
            core,
            matches.get_one::<String>("app"),
            matches.get_one::<String>("scratchpad"),
        ) {
            helpers::LaunchTarget::AppAndTarget(app, target) => {
                (app, core.resolver().get_scratchpad(target)?)
            }
            helpers::LaunchTarget::App(app) => (app, core.resolver().get_current_scratchpad()?),
            helpers::LaunchTarget::Target(target) => {
                let app = core.config().get_default_app().ok_or_else(|| errors::user(
                    "No default application available.",
                    "Make sure that you add an app to your config file using 'git-tool config add apps/bash' or similar."))?;

                (app, core.resolver().get_scratchpad(target)?)
            }
            helpers::LaunchTarget::Err(err) => return Err(err),
            helpers::LaunchTarget::None => {
                let app = core.config().get_default_app().ok_or_else(|| errors::user(
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
        let time = chrono::Local::now();
        completer.offer(&time.format("%Yw%V").to_string());

        completer.offer_many(core.config().get_apps().map(|a| a.get_name()));

        if let Ok(pads) = core.resolver().get_scratchpads() {
            completer.offer_many(pads.iter().map(|p| p.get_name()));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::*;
    use mockall::predicate::eq;

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
        let core = Core::builder()
            .with_config(cfg)
            .with_mock_resolver(|mock| {
                let temp_path = temp_path.clone();
                mock.expect_get_current_scratchpad().returning(move || {
                    Ok(Scratchpad::new(
                        "2020w07",
                        temp_path.join("scratch").join("2020w01"),
                    ))
                });
            })
            .with_mock_launcher(|mock| {
                let temp_path = temp_path.clone();
                mock.expect_run()
                    .once()
                    .withf(move |app, target| {
                        app.get_name() == "test-app"
                            && target.get_path() == temp_path.join("scratch").join("2020w01")
                    })
                    .returning(|_, _| Box::pin(async { Ok(5) }));
            })
            .build();

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
        let core = Core::builder()
            .with_config(cfg)
            .with_mock_resolver(|mock| {
                let temp_path = temp_path.clone();
                mock.expect_get_current_scratchpad().returning(move || {
                    Ok(Scratchpad::new(
                        "2020w07",
                        temp_path.join("scratch").join("2020w01"),
                    ))
                });
            })
            .with_mock_launcher(|mock| {
                let temp_path = temp_path.clone();
                mock.expect_run()
                    .once()
                    .withf(move |app, target| {
                        app.get_name() == "test-app"
                            && target.get_path() == temp_path.join("scratch").join("2020w01")
                    })
                    .returning(|_, _| Box::pin(async { Ok(0) }));
            })
            .build();

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

        let scratch_path = temp.path().join("scratch");
        let core = Core::builder()
            .with_config(cfg)
            .with_mock_resolver(|mock| {
                let scratch_path = scratch_path.clone();
                mock.expect_get_scratchpad()
                    .with(eq("2020w07"))
                    .returning(move |_| {
                        Ok(Scratchpad::new("2020w07", scratch_path.join("2020w07")))
                    });
            })
            .with_mock_launcher(|mock| {
                mock.expect_run()
                    .once()
                    .returning(|_, _| Box::pin(async { Ok(0) }));
            })
            .build();

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

        let scratch_path = temp.path().join("scratch");
        let core = Core::builder()
            .with_config(cfg)
            .with_mock_resolver(|mock| {
                let scratch_path = scratch_path.clone();
                mock.expect_get_scratchpad()
                    .with(eq("2020w07"))
                    .returning(move |_| {
                        Ok(Scratchpad::new("2020w07", scratch_path.join("2020w07")))
                    });
            })
            .with_mock_launcher(|mock| {
                mock.expect_run()
                    .once()
                    .returning(|_, _| Box::pin(async { Ok(0) }));
            })
            .build();

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

        let scratch_path = temp.path().join("scratch");
        let core = Core::builder()
            .with_config(cfg)
            .with_mock_resolver(|mock| {
                let scratch_path = scratch_path.clone();
                mock.expect_get_scratchpad()
                    .with(eq("2020w07"))
                    .returning(move |_| {
                        Ok(Scratchpad::new("2020w07", scratch_path.join("2020w07")))
                    });
            })
            .with_mock_launcher(|mock| {
                mock.expect_run().never();
            })
            .build();

        cmd.run(&core, &args).await.unwrap_or_default();
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

        let scratch_path = temp.path().join("scratch");
        let core = Core::builder()
            .with_config(cfg)
            .with_mock_resolver(|mock| {
                let scratch_path = scratch_path.clone();
                mock.expect_get_scratchpad()
                    .with(eq("2020w07"))
                    .returning(move |_| {
                        Ok(Scratchpad::new("2020w07", scratch_path.join("2020w07")))
                    });
            })
            .with_mock_launcher(|mock| {
                mock.expect_run()
                    .returning(|_, _| Box::pin(async { Ok(0) }));
            })
            .build();

        match cmd.run(&core, &args).await {
            Ok(_) => {}
            Err(err) => panic!("{}", err.message()),
        }
    }
}
