use clap::{App, SubCommand, ArgMatches, Arg};
use super::Command;
use super::core;
use super::core::Target;
use super::super::errors;
use super::async_trait;
use std::sync::Arc;

pub struct InfoCommand {

}

#[async_trait]
impl Command for InfoCommand {
    fn name(&self) -> String {
        String::from("info")
    }

    fn app<'a, 'b>(&self) -> App<'a, 'b> {
        SubCommand::with_name(self.name().as_str())
            .version("1.0")
            .about("gets the details of a specific repository")
            .alias("i")
            .after_help("Gets the details of a specific repository, either the currently open one or one provided by its name or alias.")
            .arg(Arg::with_name("repo")
                    .help("The name of the repository to get information about.")
                    .index(1))
    }
    
    async fn run<'a>(&self, core: Arc<core::Core>, matches: &ArgMatches<'a>) -> Result<i32, errors::Error> {
        let repo = match matches.value_of("repo") {
            Some(name) => core.resolver.get_best_repo(name)?,
            None => core.resolver.get_current_repo()?
        };

        println!("Name:      {}", repo.get_name());
        println!("Namespace: {}", repo.get_namespace());
        println!("Service:   {}", repo.get_domain());
        println!("Path:      {}", repo.get_path().display());

        match core.config.get_service(repo.get_domain().as_str()) {
            Some(svc) => {
                println!("");
                println!("URLs:");
                println!(" - Website:  {}", svc.get_website(&repo)?);
                println!(" - Git SSH:  {}", svc.get_git_url(&repo)?);
                println!(" - Git HTTP: {}", svc.get_http_url(&repo)?);
            },
            None => {}
        }

        Ok(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::core::{Core, Config, Repo, MockResolver};

    #[tokio::test]
    async fn run() {
        let cmd = InfoCommand{};

        let args = cmd.app().get_matches_from(vec!["info", "repo"]);

        let cfg = Config::from_str("directory: /dev").unwrap();let mut resolver = MockResolver::default();
        resolver.set_repo(Repo::new("github.com/sierrasoftworks/git-tool", std::path::PathBuf::from("/test")));

        let core = Arc::new(Core::builder()
            .with_config(&cfg)
            .with_resolver(Arc::new(resolver))
            .build());


        match cmd.run(core, &args).await {
            Ok(_) => {},
            Err(err) => {
                panic!(err.message())
            }
        }
    }
}