use super::async_trait;
use super::*;
use crate::engine::Target;
use clap::Arg;
use tracing_batteries::prelude::*;

pub struct TempCommand;
crate::command!(TempCommand);

#[async_trait]
impl CommandRunnable for TempCommand {
    fn name(&self) -> String {
        String::from("temp")
    }

    fn app(&self) -> clap::Command {
        clap::Command::new(self.name())
            .version("1.0")
            .visible_alias("t")
            .about("opens a temporary folder which will be removed when the shell is closed")
            .arg(
                Arg::new("app")
                    .help("The name of the application to launch.")
                    .index(1),
            )
            .arg(
                Arg::new("keep")
                    .long("keep")
                    .short('k')
                    .help("do not remove the temp directory when the app exits.")
                    .action(clap::ArgAction::SetTrue),
            )
    }

    #[tracing::instrument(name = "gt temp", err, skip(self, core, matches))]
    async fn run(&self, core: &Core, matches: &ArgMatches) -> Result<i32, errors::Error> {
        let app = if let Some(app) = matches.get_one::<String>("app") {
            core.config().get_app(app).ok_or_else(|| human_errors::user("The specified application does not exist.", &["Make sure you have added the application to your config file using 'git-tool config add apps/bash' or similar."]))?
        } else {
            core.config().get_default_app().ok_or_else(|| human_errors::user("No default application available.", &["Make sure that you add an app to your config file using 'git-tool config add apps/bash' or similar."]))?
        };

        let keep = matches.get_one::<bool>("keep").copied().unwrap_or_default();

        let temp = core.resolver().get_temp(keep)?;

        if keep {
            writeln!(core.output(), "temp path: {}", temp.get_path().display())?;
        }

        let status = core.launcher().run(app, &temp).await?;
        temp.close()?;

        Ok(status)
    }

    #[tracing::instrument(name = "gt complete -- gt temp", skip(self, core, completer, _matches))]
    async fn complete(&self, core: &Core, completer: &Completer, _matches: &ArgMatches) {
        completer.offer_many(core.config().get_apps().map(|a| a.get_name()));
        completer.offer("--keep");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::*;
    use std::sync::{Arc, RwLock};

    #[tokio::test]
    async fn run_no_args() {
        let cmd = TempCommand {};

        let args = cmd.app().get_matches_from(vec!["temp"]);

        let temp = tempfile::tempdir().unwrap();
        let cfg = Config::from_str(
                format!(
            "
directory: {}=

apps:
  - name: test-app
    command: test
    args:
        - '{{ .Target.Name }}'
",
            temp.path().display(),
        ))
        .unwrap();

        let temp_path = Arc::new(RwLock::new(None));
        let core = Core::builder()
            .with_config(cfg)
            .with_mock_resolver(|mock| {
                let temp_path = temp_path.clone();
                mock.expect_get_temp().returning(move |keep| {
                    let temp = TempTarget::new(keep).unwrap();
                    *temp_path.write().unwrap() = Some(temp.get_path().to_owned());
                    Ok(temp)
                });
            })
            .with_mock_launcher(|mock| {
                let temp_path = temp_path.clone();
                mock.expect_run()
                    .once()
                    .withf(move |app, target| {
                        app.get_name() == "test-app"
                            && target.get_path() == *temp_path.read().unwrap().as_ref().unwrap()
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

        assert!(
            !temp_path.read().unwrap().as_ref().unwrap().exists(),
            "the temp dir should be removed when the app exits"
        );
    }

    #[tokio::test]
    async fn run_only_app() {
        let cmd = TempCommand {};

        let args = cmd.app().get_matches_from(vec!["temp", "test-app"]);

        let temp = tempfile::tempdir().unwrap();
        let cfg = Config::from_str(
                format!(
            "
directory: {}

apps:
  - name: test-app
    command: test
    args:
        - '{{ .Target.Name }}'
",
            temp.path().display(),
        ))
        .unwrap();

        let temp_path = Arc::new(RwLock::new(None));
        let core = Core::builder()
            .with_config(cfg)
            .with_mock_resolver(|mock| {
                let temp_path = temp_path.clone();
                mock.expect_get_temp().returning(move |keep| {
                    let temp = TempTarget::new(keep).unwrap();
                    *temp_path.write().unwrap() = Some(temp.get_path().to_owned());
                    Ok(temp)
                });
            })
            .with_mock_launcher(|mock| {
                let temp_path = temp_path.clone();
                mock.expect_run()
                    .once()
                    .withf(move |app, target| {
                        app.get_name() == "test-app"
                            && target.get_path() == *temp_path.read().unwrap().as_ref().unwrap()
                    })
                    .returning(|_, _| Box::pin(async { Ok(0) }));
            })
            .build();

        cmd.assert_run_successful(&core, &args).await;

        assert!(
            !temp_path.read().unwrap().as_ref().unwrap().exists(),
            "the temp dir should be removed when the app exits"
        );
    }

    #[tokio::test]
    async fn run_unknown_app() {
        let cmd = TempCommand {};

        let args = cmd.app().get_matches_from(vec!["temp", "unknown-app"]);

        let temp = tempfile::tempdir().unwrap();
        let cfg = Config::from_str(
                format!(
            "
directory: {}

apps:
  - name: test-app
    command: test
    args:
        - '{{ .Target.Name }}'
",
            temp.path().display(),
        ))
        .unwrap();

        let core = Core::builder()
            .with_config(cfg)
            .with_mock_resolver(|mock| {
                mock.expect_get_temp()
                    .returning(|keep| Ok(TempTarget::new(keep).unwrap()));
            })
            .with_mock_launcher(|mock| {
                mock.expect_run().never();
            })
            .build();

        cmd.run(&core, &args)
            .await
            .expect_err("should fail if an unknown app is specified");
    }
}
