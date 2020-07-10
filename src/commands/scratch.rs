use super::async_trait;
use super::*;
use super::{core::Target, tasks, tasks::Task, Command};
use clap::{App, Arg, ArgMatches, SubCommand};

pub struct ScratchCommand {}

impl Command for ScratchCommand {
    fn name(&self) -> String {
        String::from("scratch")
    }

    fn app<'a, 'b>(&self) -> App<'a, 'b> {
        SubCommand::with_name(self.name().as_str())
            .version("1.0")
            .alias("s")
            .about("opens a scratchpad using an application defined in your config")
            .arg(
                Arg::with_name("app")
                    .help("The name of the application to launch.")
                    .index(1),
            )
            .arg(
                Arg::with_name("scratchpad")
                    .help("The name of the scratchpad to open.")
                    .index(2),
            )
    }
}

#[async_trait]
impl<C: Core> CommandRunnable<C> for ScratchCommand {
    async fn run<'a>(&self, core: &C, matches: &ArgMatches<'a>) -> Result<i32, errors::Error> {
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

    async fn complete<'a>(&self, core: &C, completer: &Completer, _matches: &ArgMatches<'a>) {
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
    use super::core::{Config, CoreBuilder};
    use super::*;

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

        let core = CoreBuilder::default()
            .with_config(&cfg)
            .with_mock_launcher(|l| {
                l.status = 5;
            })
            .with_mock_resolver(|_| {})
            .build();

        match cmd.run(&core, &args).await {
            Ok(status) => {
                let launches = core.launcher().launches.lock().await;
                assert_eq!(launches.len(), 1);

                let launch = &launches[0];
                assert_eq!(launch.app.get_name(), "test-app");
                assert_eq!(
                    launch.target_path,
                    temp.path().join("scratch").join("2020w01")
                );
                assert_eq!(status, 5);
            }
            Err(err) => panic!(err.message()),
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

        let core = CoreBuilder::default()
            .with_config(&cfg)
            .with_mock_launcher(|_| {})
            .with_mock_resolver(|_| {})
            .build();

        match cmd.run(&core, &args).await {
            Ok(_) => {
                let launches = core.launcher().launches.lock().await;
                assert_eq!(launches.len(), 1);

                let launch = &launches[0];
                assert_eq!(launch.app.get_name(), "test-app");
                assert_eq!(
                    launch.target_path,
                    temp.path().join("scratch").join("2020w01")
                );
            }
            Err(err) => panic!(err.message()),
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

        let core = CoreBuilder::default()
            .with_config(&cfg)
            .with_mock_launcher(|_| {})
            .with_mock_resolver(|_| {})
            .build();

        match cmd.run(&core, &args).await {
            Ok(_) => {
                let launches = core.launcher().launches.lock().await;
                assert_eq!(launches.len(), 1);

                let launch = &launches[0];
                assert_eq!(launch.app.get_name(), "test-app");
                assert_eq!(
                    launch.target_path,
                    core.config().get_scratch_directory().join("2020w07")
                );
            }
            Err(err) => panic!(err.message()),
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

        let core = CoreBuilder::default()
            .with_config(&cfg)
            .with_mock_launcher(|_| {})
            .with_mock_resolver(|_| {})
            .build();

        match cmd.run(&core, &args).await {
            Ok(_) => {
                let launches = core.launcher().launches.lock().await;
                assert_eq!(launches.len(), 1);

                let launch = &launches[0];
                assert_eq!(launch.app.get_name(), "test-app");
                assert_eq!(
                    launch.target_path,
                    temp.path().join("scratch").join("2020w07")
                );
            }
            Err(err) => panic!(err.message()),
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

        let core = CoreBuilder::default()
            .with_config(&cfg)
            .with_mock_launcher(|_| {})
            .with_mock_resolver(|_| {})
            .build();

        match cmd.run(&core, &args).await {
            Ok(_) => {
                panic!("It should not launch an app.");
            }
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

        let core = CoreBuilder::default()
            .with_config(&cfg)
            .with_mock_launcher(|_| {})
            .with_mock_resolver(|_| {})
            .build();

        match cmd.run(&core, &args).await {
            Ok(_) => {
                let launches = core.launcher().launches.lock().await;
                assert_eq!(launches.len(), 1);

                let launch = &launches[0];
                assert_eq!(launch.app.get_name(), "test-app");
                assert_eq!(
                    launch.target_path,
                    temp.path().join("scratch").join("2020w07")
                );

                assert_eq!(launch.target_path.exists(), true);

                std::fs::remove_dir(launch.target_path.clone()).unwrap();
            }
            Err(err) => panic!(err.message()),
        }
    }
}
