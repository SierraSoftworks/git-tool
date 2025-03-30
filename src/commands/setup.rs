use crate::{
    completion::get_shells,
    core::{Error, Prompter},
    fs::to_native_path,
};
use std::{io::ErrorKind, path::PathBuf, writeln};
use tracing_batteries::prelude::*;

use clap::Arg;
use directories_next::UserDirs;
use itertools::Itertools;

use super::*;

pub struct SetupCommand;
crate::command!(SetupCommand);

#[async_trait]
impl CommandRunnable for SetupCommand {
    fn name(&self) -> String {
        String::from("setup")
    }
    fn app(&self) -> clap::Command {
        clap::Command::new(self.name())
            .version("1.0")
            .about("runs the setup wizard for first time users")
            .long_about("This setup wizard will guide you through the process of getting your first Git-Tool config set up.")
            .arg(Arg::new("force")
                .long("force")
                .short('f')
                .help("Run the setup wizard even if you already have a config file.")
                .action(clap::ArgAction::SetTrue))
    }

    #[tracing::instrument(name = "gt setup", err, skip(self, core, matches))]
    async fn run(&self, core: &Core, matches: &ArgMatches) -> Result<i32, Error> {
        if core.config().file_exists() && !matches.get_flag("force") {
            Err(errors::user(
                &format!("You already have a Git-Tool config file ({}) which will not be modified.", core.config().get_config_file().unwrap().display()),
                "If you want to replace your config file, you can use `git-tool setup --force` to bypass this check."))?;
        }

        writeln!(core.output(), "Welcome to the Git-Tool setup wizard.")?;
        writeln!(core.output(), "This wizard will help you prepare your system for use with Git-Tool, including selecting your dev directory and installing auto-complete support.\n")?;

        let mut prompter = core.prompter();

        let dev_directory = self.prompt_dev_directory(core, &mut prompter)?;
        writeln!(
            core.output(),
            "\nGotcha, we'll store your projects in {}.",
            dev_directory.display()
        )?;

        let enable_telemetry = prompter
            .prompt_bool(
                "Are you happy sharing crash reports and performance telemetry with us automatically? [Y/n]: ",
                Some(true),
            )?
            .unwrap_or(true);

        let new_config = core
            .config()
            .with_dev_directory(&dev_directory)
            .with_feature_flag("telemetry", enable_telemetry);

        new_config
            .save(
                &new_config
                    .get_config_file()
                    .or_else(core::Config::default_path)
                    .ok_or_else(|| errors::system(
                        "Could not determine a default configuration file path for your system.",
                        "Set the GITTOOL_CONFIG environment variable to a valid configuration file path and try again."))?,
            )
            .await?;

        writeln!(core.output(),"\nSuccess! We've written your config to disk, now we need to configure your system to use it.")?;
        self.prompt_setup_shell(core, &mut prompter)?;

        Ok(0)
    }

    #[tracing::instrument(
        name = "gt complete -- gt setup",
        skip(self, _core, _completer, _matches)
    )]
    async fn complete(&self, _core: &Core, _completer: &Completer, _matches: &ArgMatches) {}
}

impl SetupCommand {
    fn prompt_dev_directory(&self, core: &Core, prompter: &mut Prompter) -> Result<PathBuf, Error> {
        let default_dir = match core.config().get_dev_directory().to_str() {
            Some(path) if !path.is_empty() => Some(path.to_owned()),
            _ => None,
        }
        .or_else(|| match UserDirs::new() {
            Some(dirs) => {
                let mut path = dirs.home_dir().to_path_buf();
                path.push("dev");
                path.to_str().map(|v| v.to_owned())
            }
            None => None,
        });

        let dev_dir = prompter.prompt(
            &format!(
                "Enter a directory to hold your repositories{}: ",
                default_dir
                    .clone()
                    .map(|v| format!(" [{v}]"))
                    .unwrap_or_else(|| "".into())
            ),
            |line| {
                if line.is_empty() {
                    return true;
                }

                let path = PathBuf::from(line);
                match path.canonicalize() {
                    Ok(_) => { true },
                    Err(err) if err.kind() == ErrorKind::NotFound => { true },
                    Err(err) => {
                        writeln!(core.output(), " [!] That doesn't look like a valid path to us, please try again ({err}).").unwrap_or_default();
                        false
                    }
                }
            },
        )?
        .and_then(|v| {
            if v.is_empty() {
                default_dir.clone()
            } else {
                Some(v)
            }
        })
        .ok_or_else(|| errors::user(
            "You did not enter a valid directory to store your projects in.",
            "Enter a valid path to a directory which Git-Tool can use to store your projects in.",
        ))?;

        Ok(to_native_path(dev_dir))
    }

    fn prompt_setup_shell(&self, core: &Core, prompter: &mut Prompter) -> Result<(), Error> {
        #[cfg(windows)]
        let default_shell = "powershell";
        #[cfg(target_os = "linux")]
        let default_shell = "bash";
        #[cfg(target_os = "macos")]
        let default_shell = "zsh";

        let shells = get_shells();

        let other_shells = shells
            .iter()
            .map(|s| s.get_name())
            .filter(|&s| s != default_shell)
            .join("/");

        let shell = prompter.prompt(&format!("Enter the shell you wish to configure [{}/{}]:", default_shell.to_uppercase(), other_shells), |v| {
            if v.is_empty() {
                return true;
            }

            if shells.iter().any(|s| s.get_name() == v) {
                return true;
            }

            writeln!(core.output(), " [!] That shell is not supported, please select a supported shell and try again.").unwrap_or_default();
            false
        })?.map(|v| if v.is_empty() {
            default_shell.into()
        } else {
            v
        }).unwrap_or_else(|| default_shell.into());

        writeln!(core.output())?;
        if let Some(shell) = shells.iter().find(|s| s.get_name() == shell) {
            writeln!(core.output(), "To use Git-Tool, you'll need to add the following to your shell's config file ({}):", shell.get_config_file())?;
            writeln!(core.output(), "{}", shell.get_install())?;
        } else {
            writeln!(
                core.output(),
                "Git-Tool doesn't support your current shell with native auto-completion, please let us know about this by opening a GitHub issue."
            )?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    #[cfg_attr(feature = "pure-tests", ignore)]
    async fn run() {
        let temp = tempdir().unwrap();

        let console =
            crate::console::mock_with_input(&format!("{}\ny\nzsh\n", temp.path().display()));
        let core = Core::builder()
            .with_default_config()
            .with_console(console.clone())
            .build();

        let cmd = SetupCommand {};
        let args = cmd.app().get_matches_from(vec!["setup", "--force"]);
        cmd.assert_run_successful(&core, &args).await;

        println!("{}", console);

        assert!(
            console
                .to_string()
                .contains(&format!("{}", temp.path().display())),
            "the output should contain the project directory"
        );
        assert!(
            console.to_string().contains("zsh"),
            "the output should contain the selected shell"
        );
        assert!(
            console.to_string().contains("alias gt="),
            "the output should contain the alias command"
        );
    }
}
