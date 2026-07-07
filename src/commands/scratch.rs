use super::async_trait;
use super::*;
use super::{engine::Resolver, engine::Scratchpad, engine::Target, tasks::Task};
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
            .long_about("This command launches an application within a scratchpad directory. You may specify the scratchpad name and/or an application in any order; when either is omitted, the current week's scratchpad and your default application are used.

You may also append any number of KEY=VALUE tokens to override environment variables for the launched application (for example `gt scratch shell FOO=bar`). These overrides are applied verbatim and take precedence over any environment configured for the app.")
            .arg(
                Arg::new("args")
                    .help("The app, scratchpad name, and any KEY=VALUE environment overrides to launch with (in any order).")
                    .action(clap::ArgAction::Append),
            )
    }

    #[tracing::instrument(name = "gt scratch", err, skip(self, core, matches))]
    async fn run(&self, core: &Core, matches: &ArgMatches) -> Result<i32, human_errors::Error> {
        // The current week's scratchpad is both the fallback target and the
        // context that lets a lone app-named token be treated as an application.
        let current: Scratchpad = core.resolve(())?;
        let parsed = crate::completion::parse::<Scratchpad>(core, Some(&current), matches)?;

        let app = parsed.launch_app(core)?;
        let scratchpad = &parsed.target;

        if !scratchpad.exists() {
            sequence![tasks::NewFolder {}]
                .apply_scratchpad(core, scratchpad)
                .await?;
        }

        let status = core.launcher().run(&app, scratchpad).await?;
        Ok(status)
    }

    #[tracing::instrument(
        name = "gt complete -- gt scratch",
        skip(self, core, completer, _matches)
    )]
    async fn complete(&self, core: &Core, completer: &Completer, _matches: &ArgMatches) {
        let time = chrono::Local::now();
        completer.offer(time.format("%Yw%V").to_string());

        completer.offer_many(core.config().get_apps().map(|a| a.get_name()));

        let pads: Result<Vec<Scratchpad>, _> = core.resolve_many(());
        if let Ok(pads) = pads {
            completer.offer_many(pads.iter().map(|p| p.get_name()));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::*;
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
                let current_path = temp_path.clone();
                mock.expect_get_current_scratchpad().returning(move || {
                    Ok(Scratchpad::new(
                        "2020w07",
                        current_path.join("scratch").join("2020w01"),
                    ))
                });
                let scratch_path = temp_path.clone();
                mock.expect_get_scratchpad()
                    .with(eq("2020w07"))
                    .returning(move |_| {
                        Ok(Scratchpad::new(
                            "2020w07",
                            scratch_path.join("scratch").join("2020w01"),
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
                let current_path = temp_path.clone();
                mock.expect_get_current_scratchpad().returning(move || {
                    Ok(Scratchpad::new(
                        "2020w07",
                        current_path.join("scratch").join("2020w01"),
                    ))
                });
                let scratch_path = temp_path.clone();
                mock.expect_get_scratchpad()
                    .with(eq("2020w07"))
                    .returning(move |_| {
                        Ok(Scratchpad::new(
                            "2020w07",
                            scratch_path.join("scratch").join("2020w01"),
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

        cmd.assert_run_successful(&core, &args).await;
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
                mock.expect_get_current_scratchpad()
                    .returning(|| Ok(Scratchpad::new("2020w07", "scratch/2020w07".into())));
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

        cmd.assert_run_successful(&core, &args).await;
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
                mock.expect_get_current_scratchpad()
                    .returning(|| Ok(Scratchpad::new("2020w07", "scratch/2020w07".into())));
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

        cmd.assert_run_successful(&core, &args).await;
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
                mock.expect_get_current_scratchpad()
                    .returning(|| Ok(Scratchpad::new("2020w07", "scratch/2020w07".into())));
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

        cmd.run(&core, &args)
            .await
            .expect_err("should fail if an unknown app is specified");
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
                mock.expect_get_current_scratchpad()
                    .returning(|| Ok(Scratchpad::new("2020w07", "scratch/2020w07".into())));
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

        cmd.assert_run_successful(&core, &args).await;
    }

    #[tokio::test]
    async fn run_with_env_override() {
        let cmd = ScratchCommand {};

        let args = cmd
            .app()
            .get_matches_from(vec!["scratch", "test-app", "2020w07", "FOO=bar"]);

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
                mock.expect_get_current_scratchpad()
                    .returning(|| Ok(Scratchpad::new("2020w07", "scratch/2020w07".into())));
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
                    .withf(|app, _| {
                        app.get_name() == "test-app"
                            && app
                                .get_overrides()
                                .contains(&("FOO".to_string(), "bar".to_string()))
                    })
                    .returning(|_, _| Box::pin(async { Ok(0) }));
            })
            .build();

        cmd.assert_run_successful(&core, &args).await;
    }
}
