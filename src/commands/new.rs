use clap::{App, SubCommand, ArgMatches, Arg};
use super::*;
use super::super::errors;
use crate::tasks::*;

pub struct NewCommand {

}

impl Command for NewCommand {
    fn name(&self) -> String {
        String::from("new")
    }

    fn app<'a, 'b>(&self) -> App<'a, 'b> {
        SubCommand::with_name(self.name().as_str())
            .version("1.0")
            .about("creates a new repository")
            .alias("n")
            .alias("create")
            .after_help("Creates a new repository with the provided name.")
            .arg(Arg::with_name("repo")
                    .help("The name of the repository to create.")
                    .index(1))
    }
}


#[async_trait]
impl<F: FileSource, L: Launcher, R: Resolver> CommandRun<F, L, R> for NewCommand {    
    async fn run<'a>(&self, core: &core::Core<F, L, R>, matches: &ArgMatches<'a>) -> Result<i32, errors::Error> {
        let repo = match matches.value_of("repo") {
            Some(name) => core.resolver.get_best_repo(name)?,
            None => Err(errors::user(
                "No repository name provided for creation.",
                "Please provide a repository name when calling this method: git-tool new my/repo"))?
        };

        if repo.valid() {
            return Ok(0)
        }

        let tasks = sequence![
            GitInit{},
            GitRemote{ name: "origin".to_string() },
            GitCheckout{ branch: "main".to_string() }
            // TODO: Add a task to initialize the remote repo (GitHub etc.)
        ];

        tasks.apply_repo(core, &repo).await?;

        Ok(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::core::{Core, Config, Target};

    #[tokio::test]
    async fn run_partial() {
        let cmd = NewCommand{};

        let args = cmd.app().get_matches_from(vec!["new", "test/new-repo-partial"]);

        let cfg = Config::for_dev_directory(&get_dev_dir());

        let core = Core::builder()
            .with_config(&cfg)
            .build();

        let repo = core.resolver.get_best_repo("github.com/test/new-repo-partial").unwrap();
        assert_eq!(repo.valid(), false);

        let result = cmd.run(&core, &args).await;

        let valid = repo.valid();
        std::fs::remove_dir_all(repo.get_path()).unwrap_or_default();

        match result {
            Ok(_) => {},
            Err(err) => {
                panic!(err.message())
            }
        }

        assert_eq!(valid, true);
    }

    #[tokio::test]
    async fn run_fully_qualified() {
        let cmd = NewCommand{};

        let args = cmd.app().get_matches_from(vec!["new", "github.com/test/new-repo-full"]);

        let cfg = Config::for_dev_directory(&get_dev_dir());

        let core = Core::builder()
            .with_config(&cfg)
            .build();

        let repo = core.resolver.get_best_repo("github.com/test/new-repo-full").unwrap();
        assert_eq!(repo.valid(), false);

        let result = cmd.run(&core, &args).await;

        let valid = repo.valid();
        std::fs::remove_dir_all(repo.get_path()).unwrap_or_default();

        match result {
            Ok(_) => {},
            Err(err) => {
                panic!(err.message())
            }
        }

        assert_eq!(valid, true);
    }

    fn get_dev_dir() -> std::path::PathBuf {
        std::path::PathBuf::from(file!())
            .parent()
            .and_then(|f| f.parent())
            .and_then(|f| f.parent())
            .and_then(|f| Some(f.join("test")))
            .and_then(|f| Some(f.join("devdir")))
            .unwrap()
    }
}