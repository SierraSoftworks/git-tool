use clap::{App, SubCommand, ArgMatches, Arg};
use super::Command;
use super::*;
use super::async_trait;
use online::registry::Registry;

pub struct ConfigCommand {

}

impl Command for ConfigCommand {
    fn name(&self) -> String {
        String::from("config")
    }

    fn app<'a, 'b>(&self) -> App<'a, 'b> {
        SubCommand::with_name(self.name().as_str())
            .version("1.0")
            .about("manage your Git-Tool configuration file")
            .help_message("This tool allows you to easily make changes to your Git-Tool config file.")

            .subcommand(SubCommand::with_name("list")
                .version("1.0")
                .alias("ls")
                .about("list available config templates")
                .help_message("Gets the list of config templates which are available through the Git-Tool registry."))

            .subcommand(SubCommand::with_name("add")
                .version("1.0")
                .about("adds a configuration template to your current config file")
                .help_message("Adds a configuration template from the Git-Tool online registry to your config file.")
                .arg(Arg::with_name("id")
                    .index(1)
                    .help("the id of the configuration template you want to add")
                    .required(true)))
    }
}
    
#[async_trait]
impl<C: Core> CommandRunnable<C> for ConfigCommand {
    async fn run<'a>(&self, core: &C, matches: &ArgMatches<'a>) -> Result<i32, errors::Error> {
        match matches.subcommand() {
            ("list", Some(_args)) => {
                let registry = crate::online::GitHubRegistry;

                let entries = registry.get_entries(core).await?;
                for entry in entries {
                    writeln!(core.output().writer(), "{}", entry)?;
                }
            },
            ("add", Some(args)) => {
                let id = args.value_of("id").ok_or(errors::user(
                    "You have not provided an ID for the config template you wish to add.",
                    ""))?;

                let registry = crate::online::GitHubRegistry;
                let entry = registry.get_entry(core, id).await?;

                writeln!(core.output().writer(), "Applying {}", entry.name)?;
                writeln!(core.output().writer(), "> {}", entry.description)?;

                let mut cfg = core.config().clone();
                for ec in entry.configs {
                    if ec.is_compatible() {
                        cfg = cfg.add(ec);
                    }
                }

                match cfg.get_config_file() {
                    Some(path) => {
                        tokio::fs::write(&path, cfg.to_string()?).await?;
                    },
                    None => {
                        writeln!(core.output().writer(), "{}", cfg.to_string()?)?;
                    }
                }
            },
            _ => {
                writeln!(core.output().writer(), "{}", core.config().to_string()?)?;
            }
        }

        Ok(0)
    }

    async fn complete<'a>(&self, core: &C, completer: &Completer, matches: &ArgMatches<'a>) {
        match matches.subcommand() {
            ("list", _) => {

            },
            ("add", _) => {
                let registry = online::GitHubRegistry;
                match registry.get_entries(core).await {
                    Ok(entries) => {
                        completer.offer_many(entries);
                    },
                    _ => {}
                }
            },
            _ => {
                completer.offer_many(vec!["list", "add"]);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::core::{CoreBuilder, Config};
    use clap::ArgMatches;

    #[tokio::test]
    async fn run() {
        let args = ArgMatches::default();
        let cfg = Config::from_str("directory: /dev").unwrap();
        let core = CoreBuilder::default()
            .with_config(&cfg)
            .with_mock_output()
            .build();

        let cmd = ConfigCommand{};

        match cmd.run(&core, &args).await {
            Ok(_) => {},
            Err(err) => {
                panic!(err.message())
            }
        }

        let output = core.output().to_string();
        assert!(output.contains(&core.config().to_string().unwrap()), "the output should contain the config");
    }

    #[tokio::test]
    async fn run_list() {
        let cfg = Config::from_str("directory: /dev").unwrap();
        let core = CoreBuilder::default()
        .with_config(&cfg)
        .with_mock_output()
        .build();
        
        let cmd = ConfigCommand{};
        let args = cmd.app().get_matches_from(vec!["config", "list"]);

        match cmd.run(&core, &args).await {
            Ok(_) => {},
            Err(err) => {
                panic!(err.message())
            }
        }

        let output = core.output().to_string();
        println!("{}", output);
        assert!(output.contains("apps/bash\n"), "the output should contain some apps");
        assert!(output.contains("services/github\n"), "the output should contain some services");
    }

    #[tokio::test]
    async fn run_add_no_file() {
        let cfg = Config::from_str("directory: /dev").unwrap();
        let core = CoreBuilder::default()
            .with_config(&cfg)
            .with_mock_output()
            .build();
        
        let cmd = ConfigCommand{};
        let args = cmd.app().get_matches_from(vec!["config", "add", "apps/bash"]);

        match cmd.run(&core, &args).await {
            Ok(_) => {},
            Err(err) => {
                panic!(err.message())
            }
        }

        let output = core.output().to_string();
        println!("{}", output);
        assert!(output.contains("name: bash\n"), "the output should contain the new config");
    }

    #[tokio::test]
    async fn run_add_with_file() {
        let temp = tempdir::TempDir::new("gt-commands-config").unwrap();
        tokio::fs::write(temp.path().join("config.yml"), Config::default().to_string().unwrap()).await.unwrap();

        let cfg = Config::from_file(&temp.path().join("config.yml")).unwrap();
        let core = CoreBuilder::default()
            .with_config(&cfg)
            .with_mock_output()
            .build();
        
        let cmd = ConfigCommand{};
        let args = cmd.app().get_matches_from(vec!["config", "add", "apps/bash"]);

        match cmd.run(&core, &args).await {
            Ok(_) => {},
            Err(err) => {
                panic!(err.message())
            }
        }

        let output = core.output().to_string();
        assert!(output.contains("Applying Bash\n> "), "the output should describe what is being done");

        let new_cfg = Config::from_file(&temp.path().join("config.yml")).unwrap();
        assert!(new_cfg.get_app("bash").is_some(), "the app should be added to the config file");
    }
}