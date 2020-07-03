use clap::{App, SubCommand, ArgMatches, Arg};
use super::*;
use super::super::errors;
use crate::tasks::*;

pub struct NewCommand {

}

impl Command for NewCommand {
    fn name(&self) -> String {
        "new".into()
    }

    fn app<'a, 'b>(&self) -> App<'a, 'b> {
        SubCommand::with_name(&self.name())
            .version("1.0")
            .about("creates a new repository")
            .alias("new")
            .alias("n")
            .alias("create")
            .after_help("Creates a new repository with the provided name.")
            .arg(Arg::with_name("repo")
                    .help("The name of the repository to create.")
                    .index(1))
    }
}


#[async_trait]
impl<K: KeyChain, L: Launcher, R: Resolver, O: Output> CommandRunnable<K, L, R, O> for NewCommand {    
    async fn run<'a>(&self, core: &core::Core<K, L, R, O>, matches: &ArgMatches<'a>) -> Result<i32, errors::Error> {
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

    async fn complete<'a>(&self, core: &Core<K, L, R, O>, completer: &Completer, _matches: &ArgMatches<'a>) {
        match core.resolver.get_repos() {
            Ok(repos) => {
                let mut namespaces = std::collections::HashSet::new();
                let default_svc = core.config.get_default_service().map(|s| s.get_domain()).unwrap_or_default();

                for repo in repos {
                    if repo.get_domain() == default_svc {
                        namespaces.insert(repo.get_domain() + "/");
                    }

                    namespaces.insert(format!("{}/{}/", repo.get_domain(), repo.get_namespace()));
                }

                completer.offer_many(namespaces.iter().map(|s| s.as_str()));
            },
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::core::{Core, Config};

    #[tokio::test]
    async fn run_partial() {
        let cmd = NewCommand{};

        let args = cmd.app().get_matches_from(vec!["new", "test/new-repo-partial"]);

        let temp = tempdir::TempDir::new("gt-command-new").unwrap();
        let cfg = Config::for_dev_directory(temp.path());

        let core = Core::builder()
            .with_config(&cfg)
            .build();

        let repo = core.resolver.get_best_repo("github.com/test/new-repo-partial").unwrap();
        assert_eq!(repo.valid(), false);

        cmd.run(&core, &args).await.unwrap();

        assert!(repo.valid());
    }

    #[tokio::test]
    async fn run_fully_qualified() {
        let cmd = NewCommand{};

        let args = cmd.app().get_matches_from(vec!["new", "github.com/test/new-repo-full"]);

        let temp = tempdir::TempDir::new("gt-command-new").unwrap();
        let cfg = Config::for_dev_directory(temp.path());

        let core = Core::builder()
            .with_config(&cfg)
            .build();

        let repo = core.resolver.get_best_repo("github.com/test/new-repo-full").unwrap();
        assert_eq!(repo.valid(), false);

        cmd.run(&core, &args).await.unwrap();

        assert!(repo.valid());
    }
}