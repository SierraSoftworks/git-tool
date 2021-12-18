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
                        Arg::new("full")
                            .long("full")
                            .help("prints the full initialization script for this shell")
                            .hide(true),
                    ),
            );
        }

        cmd
    }
}

#[async_trait]
impl CommandRunnable for ShellInitCommand {
    async fn run(
        &self,
        core: &Core,
        matches: &clap::ArgMatches,
    ) -> Result<i32, crate::core::Error>
where {
        let mut output = core.output();

        match matches.subcommand() {
            Some((name, matches)) => {
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

    async fn complete(&self, _core: &Core, completer: &Completer, _matches: &ArgMatches) {
        let shells = get_shells();
        completer.offer_many(shells.iter().map(|s| s.get_name()));
    }
}

#[cfg(test)]
mod tests {
    use super::core::Config;
    use super::*;

    #[tokio::test]
    async fn run() {
        let cfg = Config::default();
        let core = Core::builder().with_config(&cfg).build();

        let output = crate::console::output::mock();

        let cmd = ShellInitCommand {};
        let args = cmd.app().get_matches_from(vec!["shell-init", "powershell"]);

        match cmd.run(&core, &args).await {
            Ok(_) => {}
            Err(err) => panic!("{}", err.message()),
        }

        assert!(
            output.to_string().contains("Invoke-Expression"),
            "the output should include the setup script"
        );
    }
}
