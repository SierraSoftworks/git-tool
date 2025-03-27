use super::*;
use crate::core::Target;
use crate::tasks::*;
use clap::Arg;
use std::path::PathBuf;
use tracing_batteries::prelude::*;

pub struct CloneCommand;

crate::command!(CloneCommand);

#[async_trait]
impl CommandRunnable for CloneCommand {
    fn name(&self) -> String {
        String::from("clone")
    }

    fn app(&self) -> clap::Command {
        clap::Command::new(self.name())
            .version("1.0")
            .about("clones a repository")
            .long_about("This command clones a repository if it does not already exist in your dev directory. It works similarly to the `gt open` command, however it will not launch an application in the repository upon completion.")
            .arg(Arg::new("repo")
                    .help("The name of the repository to open.")
                    .required(true)
                    .index(1))
    }

    #[tracing::instrument(name = "gt clone", err, skip(self, core, matches))]
    async fn run(&self, core: &Core, matches: &ArgMatches) -> Result<i32, errors::Error> {
        let repo_name = matches.get_one::<String>("repo").ok_or_else(|| errors::user(
            "You didn't specify the repository you wanted to clone.",
            "Remember to specify a repository name like this: 'git-tool clone gh:sierrasoftworks/git-tool'."))?;

        if let Some(file_path) = repo_name.strip_prefix('@') {
            // Load the list of repos to clone from a file
            let file_path: PathBuf = file_path.parse().map_err(|e| {
                errors::user_with_internal(
                    "The specified file path is not valid.",
                    "Please make sure you are specifying a valid file path for your import file.",
                    e,
                )
            })?;

            let file = std::fs::read_to_string(&file_path).map_err(|e| {
                errors::user_with_internal(
                    "Could not read the specified clone file.",
                    "Please make sure the file exists and is readable.",
                    e,
                )
            })?;

            let operation = sequence![GitClone {}];

            for line in file.lines() {
                if line.trim_start().is_empty() || line.trim_start().starts_with('#') {
                    continue;
                }

                let repo = core.resolver().get_best_repo(line.trim())?;
                writeln!(core.output(), "{}", repo)?;
                match operation.apply_repo(core, &repo).await {
                    Ok(()) => {}
                    Err(e) => return Err(e),
                }
            }
        } else {
            let repo = core.resolver().get_best_repo(repo_name)?;

            if !repo.exists() {
                match sequence![GitClone {}].apply_repo(core, &repo).await {
                    Ok(()) => {}
                    Err(e) => return Err(e),
                }
            }
        }

        Ok(0)
    }

    #[tracing::instrument(
        name = "gt complete -- gt clone",
        skip(self, core, completer, _matches)
    )]
    async fn complete(&self, core: &Core, completer: &Completer, _matches: &ArgMatches) {
        completer.offer_apps(core);
        completer.offer_namespaces(core);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::*;
    use tempfile::tempdir;

    #[tokio::test]
    #[cfg_attr(feature = "pure-tests", ignore)]
    async fn run() {
        let cmd = CloneCommand {};

        let args = cmd.app().get_matches_from(vec!["clone", "repo"]);

        let cfg = Config::from_str(
            "
directory: /dev

apps:
  - name: test-app
    command: test
    args:
        - '{{ .Target.Name }}'

features:
  http_transport: true
",
        )
        .unwrap();

        let temp = tempdir().unwrap();
        std::fs::create_dir(temp.path().join("repo")).expect("create test repo dir");

        let temp_path = temp.path().to_path_buf();
        let core = Core::builder()
            .with_config(cfg)
            .with_mock_launcher(|mock| {
                mock.expect_run().never();
            })
            .with_mock_resolver(|mock| {
                let temp_path = temp_path.clone();
                mock.expect_get_best_repo()
                    .once()
                    .with(mockall::predicate::eq("repo"))
                    .returning(move |_| {
                        Ok(Repo::new("gh:git-fixtures/basic", temp_path.join("repo")))
                    });
            })
            .build();

        cmd.assert_run_successful(&core, &args).await;
    }

    #[tokio::test]
    #[cfg_attr(feature = "pure-tests", ignore)]
    async fn run_batch() {
        let cmd = CloneCommand {};

        let temp = tempdir().unwrap();
        std::fs::create_dir(temp.path().join("repo")).expect("create test repo dir");

        let args = cmd.app().get_matches_from(vec![
            "clone",
            format!("@{}", temp.path().join("import.txt").display()).as_str(),
        ]);

        let cfg = Config::from_str(
            "
    directory: /dev

    apps:
      - name: test-app
        command: test
        args:
            - '{{ .Target.Name }}'

    features:
      http_transport: true
    ",
        )
        .unwrap();

        let temp_path = temp.path().to_path_buf();

        std::fs::write(temp.path().join("import.txt"), "gh:git-fixtures/basic")
            .expect("writing should succeed");

        let core = Core::builder()
            .with_config(cfg)
            .with_mock_launcher(|mock| {
                mock.expect_run().never();
            })
            .with_mock_resolver(|mock| {
                let temp_path = temp_path.clone();
                mock.expect_get_best_repo()
                    .once()
                    .with(mockall::predicate::eq("gh:git-fixtures/basic"))
                    .returning(move |_| {
                        Ok(Repo::new("gh:git-fixtures/basic", temp_path.join("repo")))
                    });
            })
            .build();

        cmd.assert_run_successful(&core, &args).await;
    }
}
