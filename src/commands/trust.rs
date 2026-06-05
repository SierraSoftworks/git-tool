use super::*;
use crate::engine::{Config, Identifier, Repo, RepoConfig};
use crate::errors::HumanErrorResultExt;
use clap::Arg;
use tracing_batteries::prelude::*;

pub struct TrustCommand;
crate::command!(TrustCommand);

/// The decision a user can make when prompted to trust a repository's
/// configuration file.
enum TrustDecision {
    /// Trust this version of the configuration permanently (persisting it to the
    /// root configuration file).
    Always,
    /// Trust the configuration for this invocation only.
    Once,
    /// Do not trust the configuration.
    Abort,
}

#[async_trait]
impl CommandRunnable for TrustCommand {
    fn name(&self) -> String {
        String::from("trust")
    }

    fn app(&self) -> clap::Command {
        clap::Command::new(self.name())
            .version("1.0")
            .about("manages the list of repositories whose 'git-tool.yml' configuration you trust")
            .long_about("Git-Tool only runs tasks from a repository's 'git-tool.yml' file once you have confirmed that you trust its contents. This command lets you manage that trust directly: run it within (or against) a repository to trust its current configuration, use '--remove' to revoke trust, or '--list' to review which repositories you currently trust.")
            .arg(Arg::new("repo")
                    .help("The repository whose configuration you wish to trust (defaults to the current repository).")
                    .index(1))
            .arg(Arg::new("remove")
                    .short('r')
                    .long("remove")
                    .visible_alias("rm")
                    .help("remove trust for the specified repository.")
                    .action(clap::ArgAction::SetTrue))
            .arg(Arg::new("list")
                    .short('l')
                    .long("list")
                    .help("list the repositories whose configuration you currently trust.")
                    .action(clap::ArgAction::SetTrue))
    }

    #[tracing::instrument(name = "gt trust", err, skip(self, core, matches))]
    async fn run(&self, core: &Core, matches: &ArgMatches) -> Result<i32, human_errors::Error> {
        if matches.get_flag("list") {
            return self.list_trusted(core);
        }

        let repo = match matches.get_one::<String>("repo") {
            Some(value) => {
                let identifier: Identifier = value.parse()?;
                core.resolver().get_best_repo(&identifier)?
            }
            None => core.resolver().get_current_repo().map_err(|_| {
                human_errors::user(
                    "You are not currently within a repository and did not specify one to trust.",
                    &["Run this command from within a repository, or specify a repository like this: 'git-tool trust github.com/sierrasoftworks/git-tool'."],
                )
            })?,
        };

        let repo_id = repo.to_string();

        if matches.get_flag("remove") {
            let cfg = core.config().without_trusted_repo(&repo_id);
            save_config(core, &cfg).await?;
            writeln!(
                core.output(),
                "Removed trust for the repository '{repo_id}'."
            )
            .to_human_error()?;
            return Ok(0);
        }

        let config = RepoConfig::for_repo(&repo)?.ok_or_else(|| {
            human_errors::user(
                format!("The repository '{repo_id}' does not contain a 'git-tool.yml' configuration file."),
                &["Add a 'git-tool.yml' file to the root of the repository before trying to trust it."],
            )
        })?;

        let cfg = core
            .config()
            .with_trusted_repo(repo_id.clone(), config.hash()?);
        save_config(core, &cfg).await?;
        writeln!(
            core.output(),
            "Trusted the current configuration of the repository '{repo_id}'."
        )
        .to_human_error()?;

        Ok(0)
    }

    #[tracing::instrument(
        name = "gt complete -- gt trust",
        skip(self, core, completer, _matches)
    )]
    async fn complete(&self, core: &Core, completer: &Completer, _matches: &ArgMatches) {
        completer.offer("--remove");
        completer.offer("--list");
        completer.offer_aliases(core);
        completer.offer_repos(core);
    }
}

impl TrustCommand {
    fn list_trusted(&self, core: &Core) -> Result<i32, human_errors::Error> {
        let mut output = core.output();
        let mut repos: Vec<(&String, &String)> = core.config().get_trusted_repos().collect();
        repos.sort_by(|a, b| a.0.cmp(b.0));

        if repos.is_empty() {
            writeln!(output, "You have not trusted any repositories yet.").to_human_error()?;
        } else {
            for (repo, hash) in repos {
                writeln!(output, "{repo} {hash}").to_human_error()?;
            }
        }

        Ok(0)
    }
}

/// Ensures that the provided repository's configuration is trusted before its
/// tasks are executed. If the configuration has already been trusted (its hash
/// matches the entry in the root configuration) this returns `Ok(true)`
/// immediately. Otherwise the configuration is shown to the user and they are
/// prompted to decide whether to trust it. Returns `Ok(false)` if the user
/// declines to trust the repository.
#[tracing::instrument(name = "trust:ensure_trusted", err, skip(core, config))]
pub async fn ensure_trusted(
    core: &Core,
    repo: &Repo,
    config: &RepoConfig,
) -> Result<bool, human_errors::Error> {
    let repo_id = repo.to_string();

    if core.config().is_repo_trusted(&repo_id, &config.hash()?) {
        return Ok(true);
    }

    {
        let mut output = core.output();
        writeln!(
            output,
            "The repository '{repo_id}' contains a 'git-tool.yml' configuration which you have not trusted:"
        )
        .to_human_error()?;
        writeln!(output).to_human_error()?;
        writeln!(output, "{}", config.to_yaml()?).to_human_error()?;
    }

    match prompt_trust_decision(core)? {
        TrustDecision::Always => {
            let cfg = core
                .config()
                .with_trusted_repo(repo_id.clone(), config.hash()?);
            save_config(core, &cfg).await?;
            writeln!(
                core.output(),
                "Trusted the current configuration of the repository '{repo_id}'."
            )
            .to_human_error()?;
            Ok(true)
        }
        TrustDecision::Once => Ok(true),
        TrustDecision::Abort => {
            writeln!(
                core.output(),
                "Skipping tasks from the untrusted repository '{repo_id}'."
            )
            .to_human_error()?;
            Ok(false)
        }
    }
}

fn prompt_trust_decision(core: &Core) -> Result<TrustDecision, human_errors::Error> {
    let mut prompter = core.prompter();
    let answer = prompter.prompt(
        "Do you trust this repository? [a]lways / [o]nce / [N]o: ",
        |l| {
            matches!(
                l.to_lowercase().as_str(),
                "a" | "always" | "o" | "once" | "n" | "no"
            )
        },
    )?;

    Ok(match answer.map(|a| a.to_lowercase()).as_deref() {
        Some("a") | Some("always") => TrustDecision::Always,
        Some("o") | Some("once") => TrustDecision::Once,
        _ => TrustDecision::Abort,
    })
}

/// Persists the provided configuration to disk if a config file path is known,
/// otherwise prints it so the user can persist it themselves.
async fn save_config(core: &Core, config: &Config) -> Result<(), human_errors::Error> {
    match config.get_config_file() {
        Some(path) => config.save(&path).await,
        None => {
            writeln!(core.output(), "{}", config.to_string()?).to_human_error()?;
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::{Repo, Target};

    fn write_repo_config(repo: &Repo, contents: &str) {
        std::fs::create_dir_all(repo.get_path()).unwrap();
        std::fs::write(repo.get_path().join("git-tool.yml"), contents).unwrap();
    }

    const EXAMPLE_CONFIG: &str = "tasks:\n  build:\n    command: cargo\n    args:\n      - build\n";

    #[tokio::test]
    async fn ensure_trusted_when_already_trusted() {
        let temp = tempfile::tempdir().unwrap();
        let repo = Repo::new("gh:sierrasoftworks/test-trust", temp.path().join("repo"));
        write_repo_config(&repo, EXAMPLE_CONFIG);

        let config = RepoConfig::for_repo(&repo).unwrap().unwrap();

        let cfg = crate::engine::Config::for_dev_directory(temp.path())
            .with_trusted_repo(repo.to_string(), config.hash().unwrap());

        let core = Core::builder().with_config(cfg).with_null_console().build();

        assert!(ensure_trusted(&core, &repo, &config).await.unwrap());
    }

    #[tokio::test]
    async fn ensure_trusted_prompt_always_persists() {
        let temp = tempfile::tempdir().unwrap();
        let repo = Repo::new("gh:sierrasoftworks/test-trust", temp.path().join("repo"));
        write_repo_config(&repo, EXAMPLE_CONFIG);

        let config = RepoConfig::for_repo(&repo).unwrap().unwrap();

        let console = crate::console::mock_with_input("always\n");
        let core = Core::builder()
            .with_config_for_dev_directory(temp.path())
            .with_console(console.clone())
            .build();

        assert!(ensure_trusted(&core, &repo, &config).await.unwrap());
    }

    #[tokio::test]
    async fn ensure_trusted_prompt_abort() {
        let temp = tempfile::tempdir().unwrap();
        let repo = Repo::new("gh:sierrasoftworks/test-trust", temp.path().join("repo"));
        write_repo_config(&repo, EXAMPLE_CONFIG);

        let config = RepoConfig::for_repo(&repo).unwrap().unwrap();

        let console = crate::console::mock_with_input("no\n");
        let core = Core::builder()
            .with_config_for_dev_directory(temp.path())
            .with_console(console.clone())
            .build();

        assert!(!ensure_trusted(&core, &repo, &config).await.unwrap());
    }

    #[tokio::test]
    async fn ensure_trusted_prompt_once_does_not_persist() {
        let temp = tempfile::tempdir().unwrap();
        let repo = Repo::new("gh:sierrasoftworks/test-trust", temp.path().join("repo"));
        write_repo_config(&repo, EXAMPLE_CONFIG);

        let config = RepoConfig::for_repo(&repo).unwrap().unwrap();

        let console = crate::console::mock_with_input("once\n");
        let core = Core::builder()
            .with_config_for_dev_directory(temp.path())
            .with_console(console.clone())
            .build();

        assert!(ensure_trusted(&core, &repo, &config).await.unwrap());
        assert!(
            !core
                .config()
                .is_repo_trusted(&repo.to_string(), &config.hash().unwrap())
        );
    }
}
