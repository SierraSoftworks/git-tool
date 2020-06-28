use super::*;
use clap::{Arg, SubCommand};
use crate::completion::get_shells;

pub struct ShellInitCommand {

}

impl Command for ShellInitCommand {
    fn name(&self) -> String {
        String::from("shell-init")
    }
    fn app<'a, 'b>(&self) -> clap::App<'a, 'b> {
        let shells = get_shells();

        let mut cmd = SubCommand::with_name(&self.name())
            .version("1.0")
            .about("configures your shell for use with Git-Tool")
            .after_help("Used to configure your shell environment to ensure that it works correctly with Git-Tool, including auto-complete support.");

        for shell in shells {
            cmd = cmd.subcommand(SubCommand::with_name(shell.get_name())
                .about("prints the initialization script for this shell")
                .arg(Arg::with_name("full")
                    .long("full")
                    .hidden(true)));
        }

        cmd
    }
}

#[async_trait]
impl<F: FileSource, L: Launcher, R: Resolver> CommandRunnable<F, L, R> for ShellInitCommand {
    async fn run<'a>(&self, _core: &crate::core::Core<F, L, R>, matches: &clap::ArgMatches<'a>) -> Result<i32, crate::core::Error>
    where F: FileSource, L: Launcher, R: Resolver {

        let shell_name = matches.subcommand_name().ok_or(errors::user(
            "You did not provide the name of the shell you want to configure.",
            "Make sure you provide the shell name by running `git-tool shell-init powershell` or equivalent."))?;

        let shells = get_shells();
        let shell = shells.iter().find(|s| s.get_name() == shell_name).ok_or(errors::user(
            &format!("The shell '{}' is not currently supported by Git-Tool.", shell_name),
            "Make sure you're using a supported shell, or submit a PR on GitHub to add support for your shell."
        ))?;
            
        if matches.is_present("full") {
            print!("{}", shell.get_long_init());
        } else {
            print!("{}", shell.get_short_init());
        }

        Ok(0)
    }

    async fn complete<'a>(&self, _core: &Core<F, L, R>, completer: &Completer, _matches: &ArgMatches<'a>) {
        let shells = get_shells();
        completer.offer_many(shells.iter().map(|s| s.get_name()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::core::Config;

    #[tokio::test]
    async fn run() {
        
        let cfg = Config::default();
        let core = Core::builder()
        .with_config(&cfg)
        .build();
        
        let cmd = ShellInitCommand{};
        let args = cmd.app().get_matches_from(vec!["shell-init", "powershell"]);

        match cmd.run(&core, &args).await {
            Ok(_) => {},
            Err(err) => {
                panic!(err.message())
            }
        }
    }
}