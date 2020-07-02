use super::*;
use clap::{Arg, SubCommand};
use crate::{update::{GitHubSource, UpdateManager, Release}};

pub struct UpdateCommand {

}

impl Command for UpdateCommand {
    fn name(&self) -> String {
        String::from("update")
    }
    fn app<'a, 'b>(&self) -> clap::App<'a, 'b> {
        SubCommand::with_name(&self.name())
            .version("1.0")
            .about("updates Git-Tool automatically by fetching the latest release from GitHub")
            .after_help("Allows you to update Git-Tool to the latest version, or a specific version, automatically.")
            .arg(Arg::with_name("state")
                .long("update-resume-internal")
                .help("State information used to resume an update operation.")
                .hidden(true)
                .takes_value(true))
            .arg(Arg::with_name("list")
                .long("list")
                .help("Prints the list of available releases."))
            .arg(Arg::with_name("version")
                .help("The version you wish to update to. Defaults to the latest available version.")
                .index(1))
    }
}

#[async_trait]
impl<K: KeyChain, L: Launcher, R: Resolver> CommandRunnable<K, L, R> for UpdateCommand {
    async fn run<'a>(&self, _core: &crate::core::Core<K, L, R>, matches: &clap::ArgMatches<'a>) -> Result<i32, crate::core::Error>
    where K: KeyChain, L: Launcher, R: Resolver {
        let current_version: semver::Version = env!("CARGO_PKG_VERSION").parse().map_err(|err| errors::system_with_internal(
            "Could not parse the current application version into a SemVer version number.",
            "Please report this issue to us on GitHub and try updating manually by downloading the latest release from GitHub once the problem is resolved.",
            err))?;
        let manager: UpdateManager<GitHubSource> = UpdateManager::default();

        let releases = manager.get_releases().await?;

        if matches.is_present("list") {
            for release in releases {
                let mut style = " ";
                if release.version == current_version {
                    style = "*";
                }

                println!("{} {}", style, release.id);
            }

            return Ok(0)
        }

        let mut target_release = Release::get_latest(releases.iter().filter(|r| r.version > current_version));

        if let Some(target_version) = matches.value_of("version") {
            target_release = releases.iter().find(|r| r.id == target_version);
        }

        match target_release {
            Some(release) => {
                println!("Downloading update {}...", &release.id);
                if manager.update(&release).await? {
                    println!("Shutting down to complete the update operation.");
                }
            },
            None => {
                return Err(errors::user(
                    "Could not find an available update which matches your update criteria.",
                    "If you would like to switch to a specific version, ensure that it is available by running `git-tool update --list`."))
            }
        }

        Ok(0)
    }

    async fn complete<'a>(&self, _core: &Core<K, L, R>, completer: &Completer, _matches: &ArgMatches<'a>) {
        let manager: UpdateManager<GitHubSource> = UpdateManager::default();

        match manager.get_releases().await {
            Ok(releases) => {
                completer.offer_many(releases.iter().map(|r| &r.id));
            },
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::core::Config;

    #[tokio::test]
    async fn run_list() {
        
        let cfg = Config::default();
        let core = Core::builder()
        .with_config(&cfg)
        .build();
        
        let cmd = UpdateCommand{};
        let args = cmd.app().get_matches_from(vec!["update", "--list"]);

        match cmd.run(&core, &args).await {
            Ok(_) => {},
            Err(err) => {
                panic!(err.message())
            }
        }
    }
}