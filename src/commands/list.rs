use super::*;
use crate::core::Target;
use crate::search;
use clap::Arg;

pub struct ListCommand {}

impl Command for ListCommand {
    fn name(&self) -> String {
        String::from("list")
    }
    fn app<'a>(&self) -> clap::App<'a> {
        App::new(&self.name())
            .version("1.0")
            .visible_aliases(&vec!["ls", "ll"])
            .about("list your repositories")
            .after_help("Gets the list of repositories managed by Git-Tool. These repositories can be opened using the `git-tool open` command.")
            .arg(Arg::new("filter")
                .about("A filter which limits the repositories that are returned.")
                .index(1))
            .arg(Arg::new("quiet")
                .long("quiet")
                .short('q')
                .about("Prints only the name of the repository."))
            .arg(Arg::new("full")
                .long("full")
                .about("Prints detailed information about each repository.")
                .conflicts_with("quiet"))
    }
}

#[async_trait]
impl CommandRunnable for ListCommand {
    async fn run(
        &self,
        core: &Core,
        matches: &clap::ArgMatches,
    ) -> Result<i32, crate::core::Error>
where {
        let mut output = core.output();

        let filter = match matches.value_of("filter") {
            Some(name) => name,
            None => "",
        };

        let quiet = matches.is_present("quiet");
        let full = matches.is_present("full");

        let repos = core.resolver().get_repos()?;

        let mut first = true;
        for repo in search::best_matches_by(filter, repos.iter(), |r| {
            format!("{}/{}", r.get_domain(), r.get_full_name())
        }) {
            if quiet {
                writeln!(output, "{}/{}", repo.get_domain(), repo.get_full_name())?;
            } else if full {
                if !first {
                    writeln!(output, "---")?;
                }

                writeln!(
                    output,
                    "
Name:           {name}
Namespace:      {namespace}
Service:        {domain}
Path:           {path}",
                    name = repo.get_name(),
                    namespace = repo.get_namespace(),
                    domain = repo.get_domain(),
                    path = repo.get_path().display()
                )?;

                match core.config().get_service(&repo.get_domain()) {
                    Some(svc) => writeln!(
                        output,
                        "
URLs:
  - Website:    {website}
  - Git SSH:    {git_ssh}
  - Git HTTP:   {git_http}",
                        website = svc.get_website(&repo)?,
                        git_ssh = svc.get_git_url(&repo)?,
                        git_http = svc.get_http_url(&repo)?
                    )?,
                    None => {}
                };
            } else {
                match core.config().get_service(&repo.get_domain()) {
                    Some(svc) => writeln!(
                        output,
                        "{}/{} ({})",
                        repo.get_domain(),
                        repo.get_full_name(),
                        svc.get_website(&repo)?
                    )?,
                    None => writeln!(output, "{}/{}", repo.get_domain(), repo.get_full_name())?,
                };
            }

            first = false;
        }

        Ok(0)
    }

    async fn complete(&self, _core: &Core, completer: &Completer, _matches: &ArgMatches) {
        completer.offer_many(vec!["--quiet", "-q", "--full", "-f"]);
    }
}

#[cfg(test)]
mod tests {
    use super::core::*;
    use super::*;
    use crate::test::get_dev_dir;
    use mocktopus::mocking::*;
    use std::path::PathBuf;

    #[tokio::test]
    async fn run_normal() {
        let core = Core::builder()
            .with_config(&Config::for_dev_directory(&get_dev_dir()))
            .build();

        let output = crate::console::output::mock();

        let cmd = ListCommand {};

        let args = cmd.app().get_matches_from(vec!["list"]);

        match cmd.run(&core, &args).await {
            Ok(_) => {}
            Err(err) => panic!("{}", err.message()),
        }

        assert!(
            output.to_string().contains(
                "github.com/sierrasoftworks/test1 (https://github.com/sierrasoftworks/test1)\n"
            ),
            "the output should contain the repos"
        );
    }

    #[tokio::test]
    async fn run_search_full() {
        Resolver::get_repos.mock_safe(|_| {
            MockResult::Return(Ok(vec![
                Repo::new("example.com/ns1/a", PathBuf::from("/dev/example.com/ns1/a")),
                Repo::new("example.com/ns1/b", PathBuf::from("/dev/example.com/ns1/b")),
                Repo::new("example.com/ns2/c", PathBuf::from("/dev/example.com/ns2/c")),
            ]))
        });

        let core = Core::builder().build();
        crate::console::output::mock();

        let cmd = ListCommand {};
        let args = cmd.app().get_matches_from(vec!["list", "ns2", "--full"]);

        match cmd.run(&core, &args).await {
            Ok(_) => {}
            Err(err) => panic!("{}", err.message()),
        }
    }

    #[tokio::test]
    async fn run_search_quiet() {
        Resolver::get_repos.mock_safe(|_| {
            MockResult::Return(Ok(vec![
                Repo::new("example.com/ns1/a", PathBuf::from("/dev/example.com/ns1/a")),
                Repo::new("example.com/ns1/b", PathBuf::from("/dev/example.com/ns1/b")),
                Repo::new("example.com/ns2/c", PathBuf::from("/dev/example.com/ns2/c")),
            ]))
        });

        let core = Core::builder().build();
        let output = crate::console::output::mock();

        let cmd = ListCommand {};
        let args = cmd.app().get_matches_from(vec!["list", "ns1", "--quiet"]);

        match cmd.run(&core, &args).await {
            Ok(_) => {}
            Err(err) => panic!("{}", err.message()),
        }

        assert!(
            output.to_string().contains("example.com/ns1/a\n"),
            "the output should contain the first match"
        );
        assert!(
            output.to_string().contains("example.com/ns1/b\n"),
            "the output should contain the second match"
        );
    }
}
