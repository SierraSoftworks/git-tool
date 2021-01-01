use crate::{
    core::{Config, Error, Prompter},
    fs::to_native_path,
};
use std::{
    io::ErrorKind,
    path::{Path, PathBuf},
    writeln,
};

use clap::Arg;
use directories_next::{ProjectDirs, UserDirs};

use super::*;

pub struct SetupCommand {}

impl Command for SetupCommand {
    fn name(&self) -> String {
        String::from("setup")
    }
    fn app<'a>(&self) -> clap::App<'a> {
        App::new(&self.name())
            .version("1.0")
            .about("runs the setup wizard for first time users")
            .long_about("This setup wizard will guide you through the process of getting your first Git-Tool config set up.")
            .arg(Arg::new("force")
                .long("force")
                .short('f')
                .about("Run the setup wizard even if you already have a config file."))
    }
}

#[async_trait]
impl<C: Core> CommandRunnable<C> for SetupCommand {
    async fn run(&self, core: &C, matches: &clap::ArgMatches) -> Result<i32, crate::core::Error> {
        match core.config().get_config_file() {
            Some(path) if !matches.is_present("force") => {
                Err(errors::user(
                    &format!("You already have a Git-Tool config file ({}) which will not be modified.", path.display()),
                    "If you want to replace your config file, you can use `git-tool setup --force` to bypass this check."))?;
            }
            _ => {}
        };

        writeln!(
            core.output().writer(),
            "Welcome to the Git-Tool setup wizard."
        )?;
        writeln!(core.output().writer(), "This wizard will help you prepare your system for use with Git-Tool, including selecting your dev directory and installing auto-complete support.\n")?;

        let mut prompter = Prompter::new(core.input(), core.output());

        let dev_directory = self.prompt_dev_directory(core, &mut prompter)?;
        writeln!(
            core.output().writer(),
            "\nGotcha, we'll store your projects in {}.",
            dev_directory.display()
        )?;

        let config_path = self.prompt_config_path(core, &mut prompter)?;
        writeln!(
            core.output().writer(),
            "\nGotcha, we'll store your Git-Tool config in {}.",
            config_path.display()
        )?;

        let enable_telemetry = prompter
            .prompt_bool("Are you happy sharing crash reports with us automatically? [Y/n]: ")?
            .unwrap_or(true);

        match config_path.parent() {
            Some(parent) if !parent.exists() => {
                std::fs::create_dir_all(parent).map_err(|err| errors::user_with_internal(
                    &format!("We couldn't create the config file parent directory at '{}' due to a system error.", parent.display()),
                    "For access denied errors, make sure you have write permission to the location containing your config file. If you run into trouble, please create a GitHub issue and we will try to help.",
                    err
                ))?;
            }
            _ => {}
        };

        let new_config = core
            .config()
            .with_dev_directory(&dev_directory)
            .extend(Config::from_str(&format!(
                "
directory: '' # Empty directory will be ignored in extend
features:
    telemetry: {}
",
                if enable_telemetry { "true" } else { "false" }
            ))?);

        tokio::fs::write(&config_path, new_config.to_string()?).await.map_err(|err| errors::user_with_internal(
            &format!("We couldn't write the new config file to '{}' due to a system error.", config_path.display()),
            "For access denied errors, make sure you have write permission to the location containing your config file. If you run into trouble, please create a GitHub issue and we will try to help.",
            err
        ))?;

        writeln!(core.output().writer(),"\nSuccess! We've written your config to disk, now we need to configure your system to use it.")?;
        self.prompt_setup_shell(core, &config_path, &mut prompter)?;

        Ok(0)
    }

    async fn complete(&self, _core: &C, _completer: &Completer, _matches: &ArgMatches) {}
}

impl SetupCommand {
    fn prompt_dev_directory<C: Core>(
        &self,
        core: &C,
        prompter: &mut Prompter,
    ) -> Result<PathBuf, Error> {
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
                    .map(|v| format!(" [{}]", v))
                    .unwrap_or("".into())
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
                        writeln!(core.output().writer(), " [!] That doesn't look like a valid path to us, please try again ({}).", err).unwrap_or_default();
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
        .ok_or(errors::user(
            "You did not enter a valid directory to store your projects in.",
            "Enter a valid path to a directory which Git-Tool can use to store your projects in.",
        ))?;

        Ok(to_native_path(dev_dir))
    }

    fn prompt_config_path<C: Core>(
        &self,
        core: &C,
        prompter: &mut Prompter,
    ) -> Result<PathBuf, Error> {
        let default_path = core.config().get_config_file().or_else(|| {
            match ProjectDirs::from("com", "SierraSoftworks", "Git-Tool") {
                Some(dirs) => {
                    let mut path = dirs.config_dir().to_path_buf();
                    path.push("git-tool.yml");
                    Some(path)
                }
                None => None,
            }
        });

        let config_path = prompter.prompt(
            &format!(
                "Enter a path to your git-tool.yml config file{}: ",
                default_path
                    .clone()
                    .map(|v| format!(" [{}]", v.display()))
                    .unwrap_or("".into())
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
                        writeln!(core.output().writer(), " [!] That doesn't look like a valid path to us, please try again ({}).", err).unwrap_or_default();
                        false
                    }
                }
            },
        )?
        .and_then(|v| {
            if v.is_empty() {
                default_path.clone()
            } else {
                Some(PathBuf::from(v))
            }
        })
        .ok_or(errors::user(
            "You did not enter a valid directory to store your Git-Tool config in.",
            "Enter a valid path to a file where Git-Tool will store its configuration.",
        ))?;

        Ok(to_native_path(config_path))
    }

    #[cfg(windows)]
    fn prompt_setup_shell<C: Core>(
        &self,
        core: &C,
        config_path: &Path,
        _prompter: &mut Prompter,
    ) -> Result<(), Error> {
        let mut writer = core.output().writer();

        writeln!(
            writer,
            "\nStep 1: Open your PowerShell Profile file in Notepad"
        )?;
        writeln!(writer, "\n        notepad.exe $PROFILE.CurrentUserAllHosts")?;
        writeln!(writer, "\nStep 2: Add the following to it")?;
        writeln!(writer, "\n        $env:GITTOOL_CONFIG = \"{}\" # This tells Git-Tool where to find your config file", config_path.display())?;
        writeln!(writer, "        Invoke-Expression (&git-tool shell-init powershell) # This sets up auto-complete")?;
        writeln!(writer, "        New-Alias -Name gt -Value \"git-tool.exe\" # This adds the 'gt' command line alias")?;
        writeln!(writer, "\nStep 3: Save the profile file and close Notepad")?;
        writeln!(
            writer,
            "\nStep 4: Restart your terminal and try running `gt`"
        )?;

        Ok(())
    }

    #[cfg(target_os = "linux")]
    fn prompt_setup_shell<C: Core>(
        &self,
        core: &C,
        config_path: &Path,
        _prompter: &mut Prompter,
    ) -> Result<(), Error> {
        let mut writer = core.output().writer();

        writeln!(
            writer,
            "\nStep 1: Open your .bashrc profile file in your favourite editor"
        )?;
        writeln!(writer, "\n        editor ~/.bashrc")?;
        writeln!(writer, "\nStep 2: Add the following to it")?;
        writeln!(writer, "\n        export GITTOOL_CONFIG=\"{}\" # This tells Git-Tool where to find your config file", config_path.display())?;
        writeln!(
            writer,
            "        eval \"$(git-tool shell-init bash)\" # This sets up auto-complete"
        )?;
        writeln!(
            writer,
            "        alias gt=git-tool # This adds the 'gt' command line alias"
        )?;
        writeln!(
            writer,
            "\nStep 3: Save the profile file and close your editor (`ESC :wq` for VI)"
        )?;
        writeln!(
            writer,
            "\nStep 4: Restart your terminal and try running `gt`"
        )?;

        Ok(())
    }

    #[cfg(target_os = "macos")]
    fn prompt_setup_shell<C: Core>(
        &self,
        core: &C,
        config_path: &Path,
        _prompter: &mut Prompter,
    ) -> Result<(), Error> {
        let mut writer = core.output().writer();

        writeln!(
            writer,
            "\nStep 1: Open your .zshrc profile file in your favourite editor"
        )?;
        writeln!(writer, "\n        editor ~/.zshrc")?;
        writeln!(writer, "\nStep 2: Add the following to it")?;
        writeln!(writer, "\n        export GITTOOL_CONFIG=\"{}\" # This tells Git-Tool where to find your config file", config_path.display())?;
        writeln!(
            writer,
            "        eval \"$(git-tool shell-init zsh)\" # This sets up auto-complete"
        )?;
        writeln!(
            writer,
            "        alias gt=git-tool # This adds the 'gt' command line alias"
        )?;
        writeln!(
            writer,
            "\nStep 3: Save the profile file and close your editor (`ESC :wq` for VI)"
        )?;
        writeln!(
            writer,
            "\nStep 4: Restart your terminal and try running `gt`"
        )?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::core::{Config, CoreBuilder};
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn run() {
        let args = ArgMatches::default();

        let temp = tempdir().unwrap();

        let cfg = Config::default();
        let core = CoreBuilder::default()
            .with_config(&cfg)
            .with_mock_input(|i| {
                i.set_data(&format!(
                    "{}\n{}\ny\n",
                    temp.path().display(),
                    temp.path().join("config.yml").display()
                ))
            })
            .with_mock_output()
            .build();

        let cmd = SetupCommand {};
        match cmd.run(&core, &args).await {
            Ok(_) => {}
            Err(err) => panic!(err.message()),
        }

        let output = core.output().to_string();
        assert!(
            output.contains(&format!("{}", temp.path().display())),
            "the output should contain the project directory"
        );
        assert!(
            output.contains(&format!("{}", temp.path().join("config.yml").display())),
            "the output should contain the config path"
        );
        assert!(
            output.contains("Step 4: Restart your terminal"),
            "the output should contain the final setup steps"
        );
    }
}
