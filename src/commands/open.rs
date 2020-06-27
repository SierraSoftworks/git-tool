use clap::{App, SubCommand, Arg, ArgMatches};
use super::Command;
use super::*;
use crate::core::Target;
use crate::tasks::*;

pub struct OpenCommand {

}

impl Command for OpenCommand {
    fn name(&self) -> String {
        String::from("open")
    }

    fn app<'a, 'b>(&self) -> App<'a, 'b> {
        SubCommand::with_name(self.name().as_str())
            .version("1.0")
            .alias("o")
            .alias("run")
            .about("opens a repository using an application defined in your config")
            .after_help("This command launches an application defined in your configuration within the specified repository. You can specify any combination of alias, app and repo. Aliases take precedence over repos, which take precedence over apps. When specifying an app, it should appear before the repo/alias parameter. If you are already inside a repository, you can specify only an app and it will launch in the context of the current repo.
            
New applications can be configured either by making changes to your configuration, or by using the `git-tool config add` command to install them from the GitHub registry. For example, you can use `gt config add apps/bash` to configure `bash` as an available app.")
            .arg(Arg::with_name("app")
                    .help("The name of the application to launch.")
                    .index(1))
            .arg(Arg::with_name("repo")
                    .help("The name of the repository to open.")
                    .index(2))
    }
}


#[async_trait]
impl<F: FileSource, L: Launcher, R: Resolver> CommandRun<F, L, R> for OpenCommand {    
    async fn run<'a>(&self, core: &core::Core<F, L, R>, matches: &ArgMatches<'a>) -> Result<i32, errors::Error> {
        let mut repo: Option<core::Repo> = None;
        let mut app: Option<&core::App> = core.config.get_default_app();

        match matches.value_of("repo") {
            Some(name) => {
                repo = Some(core.resolver.get_best_repo(name)?);
            },
            None => {}
        }

        match matches.value_of("app") {
            Some(name) => {
                match repo {
                    Some(_) => {
                        app = core.config.get_app(name);

                        match app {
                            Some(_) => {},
                            None => return Err(errors::user(
                                format!("Could not find application with name '{}'.", name).as_str(),
                                format!("Make sure that you are using an application which is present in your configuration file, or install it with 'git-tool config add apps/{}'.", name).as_str()))
                        }
                    }
                    None => {
                        repo = Some(core.resolver.get_best_repo(name)?);
                    }
                }
            },
            None => 
                return Err(errors::user(
                    "You did not specify the name of a repository to use.",
                    "Remember to specify a repository name like this: 'git-tool open github.com/sierrasoftworks/git-tool'."))
            
        }

        match app {
            Some(_) => {},
            None => 
                return Err(errors::user(
                    "No default application available.",
                    "Make sure that you add an app to your config file using 'git-tool config add apps/bash' or similar."))
        }

        match repo {
            Some(_) => {}
            None => {
                repo = Some(core.resolver.get_current_repo()?);
            }
        }

        if let Some(repo) = repo {
            if !repo.exists() {
                sequence![
                    GitClone{}
                ].apply_repo(core, &repo).await?;
            }

            if let Some(app) = app {
                let status = core.launcher.run(app, &repo).await?;
                return Ok(status)
            }
        }
        
        Ok(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::core::{Core, Config, Repo};
    use tempdir::TempDir;

    #[tokio::test]
    async fn run() {
        let cmd = OpenCommand{};

        let args = cmd.app().get_matches_from(vec!["open", "test-app", "repo"]);

        let cfg = Config::from_str("
directory: /dev

apps:
  - name: test-app
    command: test
    args:
        - '{{ .Target.Name }}'

features:
  http_transport: true
").unwrap();

        let temp = TempDir::new("gt-commands-open").unwrap();
        let core = Core::builder()
            .with_config(&cfg)
            .with_mock_launcher(|l| {
                l.status = 5;
            })
            .with_mock_resolver(|r| {
                r.set_repo(Repo::new("github.com/git-fixtures/basic", temp.path().join("repo").into()));
            })
            .build();
        

        match cmd.run(&core, &args).await {
            Ok(status) => {
                assert_eq!(status, 5);
                let launches = core.launcher.launches.lock().await;
                assert!(launches.len() == 1);

                let launch = &launches[0];
                assert_eq!(launch.target_path, temp.path().join("repo"))
            },
            Err(err) => {
                panic!(err.message())
            }
        }
    }
}