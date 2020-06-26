use clap::{App, SubCommand, Arg, ArgMatches};
use super::{Command, core::Target, tasks, tasks::Task};
use super::*;
use super::async_trait;

pub struct ScratchCommand {

}

impl Command for ScratchCommand {
    fn name(&self) -> String {
        String::from("scratch")
    }

    fn app<'a, 'b>(&self) -> App<'a, 'b> {
        SubCommand::with_name(self.name().as_str())
            .version("1.0")
            .alias("s")
            .about("opens a scratchpad using an application defined in your config")
            .arg(Arg::with_name("app")
                    .help("The name of the application to launch.")
                    .index(1))
            .arg(Arg::with_name("scratchpad")
                    .help("The name of the scratchpad to open.")
                    .index(2))
    }
}

#[async_trait]
impl<F: FileSource, L: Launcher, R: Resolver> CommandRun<F, L, R> for ScratchCommand {
    async fn run<'a>(&self, core: &core::Core<F, L, R>, matches: &ArgMatches<'a>) -> Result<i32, errors::Error> {
        let mut scratchpad: Option<core::Scratchpad> = None;
        let mut app: Option<&core::App> = core.config.get_default_app();

        match app {
            Some(_) => {},
            None => 
                return Err(errors::user(
                    "No default application available.",
                    "Make sure that you add an app to your config file using 'git-tool config add apps/bash' or similar."))
        }

        match matches.value_of("scratchpad") {
            Some(name) => {
                scratchpad = Some(core.resolver.get_scratchpad(name)?);
            },
            None => {}
        }

        match matches.value_of("app") {
            Some(name) => {
                match core.config.get_app(name) {
                    Some(a) => {
                        app = Some(a);
                    },
                    None => {
                        match scratchpad {
                            Some(_) => {
                                return Err(errors::user(
                                    format!("Could not find application with name '{}'.", name).as_str(),
                                    format!("Make sure that you are using an application which is present in your configuration file, or install it with 'git-tool config add apps/{}'.", name).as_str()))
                            },
                            None => {
                                scratchpad = Some(core.resolver.get_scratchpad(name)?);
                            }
                        }
                    }
                }
            },
            None => {}
        }

        match scratchpad {
            Some(_) => {},
            None => {
                scratchpad = Some(core.resolver.get_current_scratchpad()?);
            }
        }

        if let Some(scratchpad) = scratchpad {
            if let Some(app) = app {
                if !scratchpad.exists() {
                    let task = tasks::NewFolder{};
                    task.apply_scratchpad(&core, &scratchpad).await?;
                }

                let status = core.launcher.run(app, &scratchpad).await?;
                return Ok(status)
            }
        }
        
        Err(errors::system(
            "We got ourselves into an unexpected state and weren't able to open your scratchpad.",
            "Please open a bug report with us on GitHub explaining what you were doing when this happened."))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::core::{Core, Config, MockLauncher};
    use std::sync::Arc;

    #[tokio::test]
    async fn run_no_args() {
        let cmd = ScratchCommand{};

        let args = cmd.app().get_matches_from(vec!["scratch"]);

        let temp = tempdir::TempDir::new("gt-commands-scratch").unwrap();
        let cfg = Config::from_str(&format!("
directory: {}
scratchpads: {}

apps:
  - name: test-app
    command: test
    args:
        - '{{ .Target.Name }}'
", temp.path().display(), temp.path().join("scratch").display())).unwrap();

        let launcher = Arc::new(MockLauncher::default());
        
        {
            let mut status = launcher.status.lock().await;
            *status = 5;
        }

        let core = Core::builder()
            .with_config(&cfg)
            .with_launcher(launcher.clone())
            .with_mock_resolver()
            .build();


        match cmd.run(&core, &args).await {
            Ok(status) => {
                let launches = launcher.launches.lock().await;
                assert_eq!(launches.len(), 1);

                let launch = &launches[0];
                assert_eq!(launch.app.get_name(), "test-app");
                assert_eq!(launch.target_path, temp.path().join("scratch").join("2020w01"));
                assert_eq!(status, 5);
            },
            Err(err) => {
                panic!(err.message())
            }
        }
    }
    
    #[tokio::test]
    async fn run_only_app() {
        let cmd = ScratchCommand{};

        let args = cmd.app().get_matches_from(vec!["scratch", "test-app"]);

        let temp = tempdir::TempDir::new("gt-commands-scratch").unwrap();
        let cfg = Config::from_str(&format!("
directory: {}
scratchpads: {}

apps:
  - name: test-app
    command: test
    args:
        - '{{ .Target.Name }}'
", temp.path().display(), temp.path().join("scratch").display())).unwrap();

        let launcher = Arc::new(MockLauncher::default());

        let core = Core::builder()
            .with_config(&cfg)
            .with_launcher(launcher.clone())
            .with_mock_resolver()
            .build();


        match cmd.run(&core, &args).await {
            Ok(_) => {
                let launches = launcher.launches.lock().await;
                assert_eq!(launches.len(), 1);

                let launch = &launches[0];
                assert_eq!(launch.app.get_name(), "test-app");
                assert_eq!(launch.target_path, temp.path().join("scratch").join("2020w01"));
            },
            Err(err) => {
                panic!(err.message())
            }
        }
    }
    
    #[tokio::test]
    async fn run_only_scratchpad() {
        let cmd = ScratchCommand{};

        let args = cmd.app().get_matches_from(vec!["scratch", "2020w07"]);

        let launcher = Arc::new(MockLauncher::default());

        let temp = tempdir::TempDir::new("gt-commands-scratch").unwrap();
        let cfg = Config::from_str(&format!("
directory: {}
scratchpads: {}

apps:
  - name: test-app
    command: test
    args:
        - '{{ .Target.Name }}'
        ", temp.path().display(), temp.path().join("scratch").display())).unwrap();

        let core = Core::builder()
            .with_config(&cfg)
            .with_launcher(launcher.clone())
            .with_mock_resolver()
            .build();

        match cmd.run(&core, &args).await {
            Ok(_) => {
                let launches = launcher.launches.lock().await;
                assert_eq!(launches.len(), 1);

                let launch = &launches[0];
                assert_eq!(launch.app.get_name(), "test-app");
                assert_eq!(launch.target_path, core.config.get_scratch_directory().join("2020w07"));
            },
            Err(err) => {
                panic!(err.message())
            }
        }
    }
    
    #[tokio::test]
    async fn run_app_and_scratchpad() {
        let cmd = ScratchCommand{};

        let args = cmd.app().get_matches_from(vec!["scratch", "test-app", "2020w07"]);

        let temp = tempdir::TempDir::new("gt-commands-scratch").unwrap();
        let cfg = Config::from_str(&format!("
directory: {}
scratchpads: {}

apps:
  - name: test-app
    command: test
    args:
        - '{{ .Target.Name }}'
", temp.path().display(), temp.path().join("scratch").display())).unwrap();

        let launcher = Arc::new(MockLauncher::default());

        let core = Core::builder()
            .with_config(&cfg)
            .with_launcher(launcher.clone())
            .with_mock_resolver()
            .build();


        match cmd.run(&core, &args).await {
            Ok(_) => {
                let launches = launcher.launches.lock().await;
                assert_eq!(launches.len(), 1);
                
                let launch = &launches[0];
                assert_eq!(launch.app.get_name(), "test-app");
                assert_eq!(launch.target_path, temp.path().join("scratch").join("2020w07"));
            },
            Err(err) => {
                panic!(err.message())
            }
        }
    }
    
    #[tokio::test]
    async fn run_unknown_app() {
        let cmd = ScratchCommand{};

        let args = cmd.app().get_matches_from(vec!["scratch", "unknown-app", "2020w07"]);

        let temp = tempdir::TempDir::new("gt-commands-scratch").unwrap();
        let cfg = Config::from_str(&format!("
directory: {}
scratchpads: {}

apps:
  - name: test-app
    command: test
    args:
        - '{{ .Target.Name }}'
", temp.path().display(), temp.path().join("scratch").display())).unwrap();

        let launcher = Arc::new(MockLauncher::default());

        let core = Core::builder()
            .with_config(&cfg)
            .with_launcher(launcher.clone())
            .with_mock_resolver()
            .build();


        match cmd.run(&core, &args).await {
            Ok(_) => {
                panic!("It should not launch an app.");
            },
            Err(_) => {}
        }
    }
    
    #[tokio::test]
    async fn run_new_scratchpad() {
        let cmd = ScratchCommand{};

        let args = cmd.app().get_matches_from(vec!["scratch", "2020w07"]);

        let temp = tempdir::TempDir::new("gt-commands-scratch").unwrap();
        let cfg = Config::from_str(&format!("
directory: {}
scratchpads: {}

apps:
  - name: test-app
    command: test
    args:
        - '{{ .Target.Name }}'
", temp.path().display(), temp.path().join("scratch").display())).unwrap();

        let launcher = Arc::new(MockLauncher::default());

        let core = Core::builder()
            .with_config(&cfg)
            .with_launcher(launcher.clone())
            .with_mock_resolver()
            .build();


        match cmd.run(&core, &args).await {
            Ok(_) => {
                let launches = launcher.launches.lock().await;
                assert_eq!(launches.len(), 1);

                let launch = &launches[0];
                assert_eq!(launch.app.get_name(), "test-app");
                assert_eq!(launch.target_path, temp.path().join("scratch").join("2020w07"));

                assert_eq!(launch.target_path.exists(), true);

                std::fs::remove_dir(launch.target_path.clone()).unwrap();
            },
            Err(err) => {
                panic!(err.message())
            }
        }
    }
}