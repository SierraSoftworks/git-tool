use super::*;
use clap::{Arg, SubCommand};
use crate::search;
use crate::core::Target;

pub struct ListCommand {

}

impl Command for ListCommand {
    fn name(&self) -> String {
        String::from("list")
    }
    fn app<'a, 'b>(&self) -> clap::App<'a, 'b> {
        SubCommand::with_name(&self.name())
            .version("1.0")
            .alias("ls")
            .alias("ll")
            .about("list your repositories")
            .after_help("Gets the list of repositories managed by Git-Tool. These repositories can be opened using the `git-tool open` command.")
            .arg(Arg::with_name("filter")
                .help("A filter which limits the repositories that are returned.")
                .index(1))
            .arg(Arg::with_name("quiet")
                .long("quiet")
                .short("q")
                .help("Prints only the name of the repository."))
            .arg(Arg::with_name("full")
                .long("full")
                .help("Prints detailed information about each repository.")
                .conflicts_with("quiet"))
                
    }
}


#[async_trait]
impl<F: FileSource, L: Launcher, R: Resolver> CommandRun<F, L, R> for ListCommand {
    async fn run<'a>(&self, core: &crate::core::Core<F, L, R>, matches: &clap::ArgMatches<'a>) -> Result<i32, crate::core::Error>
    where F: FileSource, L: Launcher, R: Resolver {
        let filter = match matches.value_of("filter") {
            Some(name) => name,
            None => ""
        };

        let quiet = matches.is_present("quiet");
        let full = matches.is_present("full");

        let repos = core.resolver.get_repos()?;

        let mut first = true;
        for repo in repos.iter().filter(|r| search::matches(&format!("{}/{}", r.get_domain(), r.get_full_name()), filter)) {
            if quiet {
                println!("{}/{}", repo.get_domain(), repo.get_full_name());
            } else if full {
                if !first {
                    println!("---");
                }

                println!("
Name:           {name}
Namespace:      {namespace}
Service:        {domain}
Path:           {path}", 
                name=repo.get_name(),
                namespace=repo.get_namespace(),
                domain=repo.get_domain(),
                path=repo.get_path().display());

                match core.config.get_service(&repo.get_domain()) {
                    Some(svc) => println!("
URLs:
  - Website:    {website}
  - Git SSH:    {git_ssh}
  - Git HTTP:   {git_http}", 
                website=svc.get_website(&repo)?,
                git_ssh=svc.get_git_url(&repo)?,
                git_http=svc.get_http_url(&repo)?),
                    None => {}
                };
            } else {
                match core.config.get_service(&repo.get_domain()) {
                    Some(svc) => println!("{}/{} ({})", repo.get_domain(), repo.get_full_name(), svc.get_website(&repo)?),
                    None => println!("{}/{}", repo.get_domain(), repo.get_full_name())
                };
            }

            first = false;
        }

        Ok(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::core::Config;
    use crate::test::get_dev_dir;

    #[tokio::test]
    async fn run_normal() {
        let core = Core::builder()
            .with_config(&Config::for_dev_directory(&get_dev_dir()))
            .build();
        
        let cmd = ListCommand{};
        
        let args = cmd.app().get_matches_from(vec!["list"]);

        match cmd.run(&core, &args).await {
            Ok(_) => {},
            Err(err) => {
                panic!(err.message())
            }
        }
    }
}