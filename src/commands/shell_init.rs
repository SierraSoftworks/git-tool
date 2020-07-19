use super::*;
use crate::completion::get_shells;
use clap::Arg;

pub struct ShellInitCommand {}

impl Command for ShellInitCommand {
    fn name(&self) -> String {
        String::from("shell-init")
    }
    fn app<'a>(&self) -> clap::App<'a> {
        let shells = get_shells();

        let mut cmd = App::new(&self.name())
            .version("1.0")
            .about("configures your shell for use with Git-Tool")
            .long_about("Used to configure your shell environment to ensure that it works correctly with Git-Tool, including auto-complete support.");

        for shell in shells {
            cmd = cmd.subcommand(
                App::new(shell.get_name())
                    .about("prints the initialization script for this shell")
                    .arg(
                        Arg::with_name("full")
                            .long("full")
                            .about("prints the full initialization script for this shell")
                            .hidden(true),
                    ),
            );
        }

        cmd
    }
}

#[async_trait]
impl<C: Core> CommandRunnable<C> for ShellInitCommand {
    async fn run(&self, core: &C, matches: &clap::ArgMatches) -> Result<i32, crate::core::Error>
    where
        C: Core,
    {
        let mut output = core.output().writer();

        match matches.subcommand() {
            (name, Some(matches)) => {
                let shells = get_shells();
                let shell = shells.iter().find(|s| s.get_name() == name).ok_or(errors::user(
                    &format!("The shell '{}' is not currently supported by Git-Tool.", name),
                    "Make sure you're using a supported shell, or submit a PR on GitHub to add support for your shell."
                ))?;

                if matches.is_present("full") {
                    write!(output, "{}", shell.get_long_init())?;
                } else {
                    write!(output, "{}", shell.get_short_init())?;
                }
            }
            _ => {
                Err(errors::user(
                    "You did not provide the name of the shell you want to configure.",
                    "Make sure you provide the shell name by running `git-tool shell-init powershell` or equivalent."))?;
            }
        }

        Ok(0)
    }

    async fn complete(&self, _core: &C, completer: &Completer, _matches: &ArgMatches) {
        let shells = get_shells();
        completer.offer_many(shells.iter().map(|s| s.get_name()));
    }
}

#[cfg(test)]
mod tests {
    use super::core::{Config, CoreBuilder};
    use super::*;

    #[tokio::test]
    async fn run() {
        let cfg = Config::default();
        let core = CoreBuilder::default()
            .with_config(&cfg)
            .with_mock_output()
            .build();

        let cmd = ShellInitCommand {};
        let args = cmd.app().get_matches_from(vec!["shell-init", "powershell"]);

        match cmd.run(&core, &args).await {
            Ok(_) => {}
            Err(err) => panic!(err.message()),
        }

        let output = core.output().to_string();
        assert!(
            output.contains("Invoke-Expression"),
            "the output should include the setup script"
        );
    }
}
