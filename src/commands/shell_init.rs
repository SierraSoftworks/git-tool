use super::*;
use crate::completion::get_shells;
use clap::Arg;
use tracing_batteries::prelude::*;

pub struct ShellInitCommand;
crate::command!(ShellInitCommand);

#[async_trait]
impl CommandRunnable for ShellInitCommand {
    fn name(&self) -> String {
        String::from("shell-init")
    }
    fn app(&self) -> clap::Command {
        let shells = get_shells();

        let mut cmd = clap::Command::new(self.name())
            .version("1.0")
            .about("configures your shell for use with Git-Tool")
            .long_about("Used to configure your shell environment to ensure that it works correctly with Git-Tool, including auto-complete support.");

        for shell in shells {
            cmd = cmd.subcommand(
                clap::Command::new(shell.get_name().to_owned())
                    .about("prints the initialization script for this shell")
                    .arg(
                        Arg::new("full")
                            .long("full")
                            .help("prints the full initialization script for this shell")
                            .hide(true)
                            .action(clap::ArgAction::SetTrue),
                    ),
            );
        }

        cmd
    }

    #[tracing::instrument(name = "gt shell-init", err, skip(self, core, matches))]
    async fn run(&self, core: &Core, matches: &ArgMatches) -> Result<i32, core::Error>
where {
        let mut output = core.output();

        match matches.subcommand() {
            Some((name, matches)) => {
                let shells = get_shells();
                let shell = shells.iter().find(|s| s.get_name() == name).ok_or_else(|| errors::user(
                    &format!("The shell '{name}' is not currently supported by Git-Tool."),
                    "Make sure you're using a supported shell, or submit a PR on GitHub to add support for your shell."
                ))?;

                if matches.get_flag("full") {
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
    #[tracing::instrument(
        name = "gt complete -- gt shell-init",
        skip(self, _core, completer, _matches)
    )]
    async fn complete(&self, _core: &Core, completer: &Completer, _matches: &ArgMatches) {
        let shells = get_shells();
        completer.offer_many(shells.iter().map(|s| s.get_name()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::console::MockConsoleProvider;
    use std::sync::Arc;

    #[tokio::test]
    async fn run() {
        let console = Arc::new(MockConsoleProvider::new());
        let core = Core::builder()
            .with_default_config()
            .with_console(console.clone())
            .build();

        let cmd = ShellInitCommand {};
        let args = cmd.app().get_matches_from(vec!["shell-init", "powershell"]);

        match cmd.run(&core, &args).await {
            Ok(_) => {}
            Err(err) => panic!("{}", err.message()),
        }

        assert!(
            console.to_string().contains("Invoke-Expression"),
            "the output should include the setup script"
        );
    }
}
