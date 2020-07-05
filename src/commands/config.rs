use super::async_trait;
use super::Command;
use super::*;
use clap::{App, Arg, ArgMatches, SubCommand};
use online::registry::Registry;

pub struct ConfigCommand {}

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

            .subcommand(SubCommand::with_name("alias")
                .version("1.0")
                .about("manage aliases for your repositories")
                .help_message("Set or remove aliases for your repositories within your config file.")
                .arg(Arg::with_name("delete")
                    .short("-d")
                    .long("--delete")
                    .help("delete the alias from your config file"))
                .arg(Arg::with_name("alias")
                    .help("the name of the alias to manage")
                    .index(1))
                .arg(Arg::with_name("repo")
                    .help("the fully qualified repository name")
                    .index(2)))
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
            }
            ("add", Some(args)) => {
                let id = args.value_of("id").ok_or(errors::user(
                    "You have not provided an ID for the config template you wish to add.",
                    "",
                ))?;

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
                    }
                    None => {
                        writeln!(core.output().writer(), "{}", cfg.to_string()?)?;
                    }
                }
            }
            ("alias", Some(args)) => match args.value_of("alias") {
                Some(alias) => {
                    if args.is_present("delete") {
                        let mut cfg = core.config().clone();
                        cfg.remove_alias(alias);

                        match cfg.get_config_file() {
                            Some(path) => {
                                tokio::fs::write(&path, cfg.to_string()?).await?;
                            }
                            None => {
                                writeln!(core.output().writer(), "{}", cfg.to_string()?)?;
                            }
                        }

                        return Ok(0);
                    }

                    match args.value_of("repo") {
                        Some(repo) => {
                            let mut cfg = core.config().clone();
                            cfg.add_alias(alias, repo);

                            match cfg.get_config_file() {
                                Some(path) => {
                                    tokio::fs::write(&path, cfg.to_string()?).await?;
                                }
                                None => {
                                    writeln!(core.output().writer(), "{}", cfg.to_string()?)?;
                                }
                            }
                        }
                        None => {
                            let mut output = core.output().writer();
                            match core.config().get_alias(alias) {
                                Some(repo) => {
                                    writeln!(output, "{} = {}", alias, repo)?;
                                }
                                None => {
                                    writeln!(output, "No alias exists with the name '{}'", alias)?;
                                }
                            }
                        }
                    }
                }
                None => {
                    let mut output = core.output().writer();
                    for (alias, repo) in core.config().get_aliases() {
                        writeln!(output, "{} = {}", alias, repo)?;
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
            ("list", _) => {}
            ("add", _) => {
                let registry = online::GitHubRegistry;
                match registry.get_entries(core).await {
                    Ok(entries) => {
                        completer.offer_many(entries);
                    }
                    _ => {}
                }
            }
            ("alias", Some(args)) => {
                if !args.is_present("alias") {
                    completer.offer_many(core.config().get_aliases().map(|(a, _)| a));
                } else {
                    if !args.is_present("delete") && !args.is_present("repo") {
                        completer.offer("-d");
                        match core.resolver().get_repos() {
                            Ok(repos) => {
                                completer.offer_many(
                                    repos.iter().map(|r| {
                                        format!("{}/{}", r.get_domain(), r.get_full_name())
                                    }),
                                );
                            }
                            _ => {}
                        }
                    }
                }
            }
            _ => {
                completer.offer_many(vec!["list", "add", "alias"]);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::core::{Config, CoreBuilder};
    use super::*;
    use crate::test::get_dev_dir;
    use clap::ArgMatches;
    use complete::helpers::test_completions_with_config;

    #[tokio::test]
    async fn run() {
        let args = ArgMatches::default();
        let cfg = Config::from_str("directory: /dev").unwrap();
        let core = CoreBuilder::default()
            .with_config(&cfg)
            .with_mock_output()
            .build();

        let cmd = ConfigCommand {};

        match cmd.run(&core, &args).await {
            Ok(_) => {}
            Err(err) => panic!(err.message()),
        }

        let output = core.output().to_string();
        assert!(
            output.contains(&core.config().to_string().unwrap()),
            "the output should contain the config"
        );
    }

    #[tokio::test]
    async fn run_list() {
        let cfg = Config::from_str("directory: /dev").unwrap();
        let core = CoreBuilder::default()
            .with_config(&cfg)
            .with_mock_output()
            .build();

        let cmd = ConfigCommand {};
        let args = cmd.app().get_matches_from(vec!["config", "list"]);

        match cmd.run(&core, &args).await {
            Ok(_) => {}
            Err(err) => panic!(err.message()),
        }

        let output = core.output().to_string();
        println!("{}", output);
        assert!(
            output.contains("apps/bash\n"),
            "the output should contain some apps"
        );
        assert!(
            output.contains("services/github\n"),
            "the output should contain some services"
        );
    }

    #[tokio::test]
    async fn run_add_no_file() {
        let cfg = Config::from_str("directory: /dev").unwrap();
        let core = CoreBuilder::default()
            .with_config(&cfg)
            .with_mock_output()
            .build();

        let cmd = ConfigCommand {};
        let args = cmd
            .app()
            .get_matches_from(vec!["config", "add", "apps/bash"]);

        match cmd.run(&core, &args).await {
            Ok(_) => {}
            Err(err) => panic!(err.message()),
        }

        let output = core.output().to_string();
        println!("{}", output);
        assert!(
            output.contains("name: bash\n"),
            "the output should contain the new config"
        );
    }

    #[tokio::test]
    async fn run_add_with_file() {
        let temp = tempfile::tempdir().unwrap();
        tokio::fs::write(
            temp.path().join("config.yml"),
            Config::default().to_string().unwrap(),
        )
        .await
        .unwrap();

        let cfg = Config::from_file(&temp.path().join("config.yml")).unwrap();
        let core = CoreBuilder::default()
            .with_config(&cfg)
            .with_mock_output()
            .build();

        let cmd = ConfigCommand {};
        let args = cmd
            .app()
            .get_matches_from(vec!["config", "add", "apps/bash"]);

        match cmd.run(&core, &args).await {
            Ok(_) => {}
            Err(err) => panic!(err.message()),
        }

        let output = core.output().to_string();
        assert!(
            output.contains("Applying Bash\n> "),
            "the output should describe what is being done"
        );

        let new_cfg = Config::from_file(&temp.path().join("config.yml")).unwrap();
        assert!(
            new_cfg.get_app("bash").is_some(),
            "the app should be added to the config file"
        );
    }

    #[tokio::test]
    async fn run_alias_list() {
        let cfg = Config::from_str(
            r#"
directory: /dev

aliases:
  test1: example.com/tests/test1
  test2: example.com/tests/test2
"#,
        )
        .unwrap();
        let core = CoreBuilder::default()
            .with_config(&cfg)
            .with_mock_output()
            .build();

        let cmd = ConfigCommand {};
        let args = cmd.app().get_matches_from(vec!["config", "alias"]);

        match cmd.run(&core, &args).await {
            Ok(_) => {}
            Err(err) => panic!(err.message()),
        }

        let output = core.output().to_string();
        println!("{}", output);
        assert!(
            output.contains("test1 = example.com/tests/test1\n"),
            "the output should contain the aliases"
        );
        assert!(
            output.contains("test2 = example.com/tests/test2\n"),
            "the output should contain the aliases"
        );
    }

    #[tokio::test]
    async fn run_alias_info() {
        let cfg = Config::from_str(
            r#"
directory: /dev

aliases:
  test1: example.com/tests/test1
  test2: example.com/tests/test2
"#,
        )
        .unwrap();
        let core = CoreBuilder::default()
            .with_config(&cfg)
            .with_mock_output()
            .build();

        let cmd = ConfigCommand {};
        let args = cmd.app().get_matches_from(vec!["config", "alias", "test1"]);

        match cmd.run(&core, &args).await {
            Ok(_) => {}
            Err(err) => panic!(err.message()),
        }

        let output = core.output().to_string();
        println!("{}", output);
        assert!(
            output.contains("test1 = example.com/tests/test1\n"),
            "the output should contain the alias"
        );
    }

    #[tokio::test]
    async fn run_alias_add() {
        let temp = tempfile::tempdir().unwrap();
        tokio::fs::write(
            temp.path().join("config.yml"),
            Config::default().to_string().unwrap(),
        )
        .await
        .unwrap();

        let cfg = Config::from_file(&temp.path().join("config.yml")).unwrap();

        let core = CoreBuilder::default()
            .with_config(&cfg)
            .with_mock_output()
            .build();

        let cmd = ConfigCommand {};
        let args =
            cmd.app()
                .get_matches_from(vec!["config", "alias", "test", "example.com/tests/test"]);

        match cmd.run(&core, &args).await {
            Ok(_) => {}
            Err(err) => panic!(err.message()),
        }

        let new_cfg = Config::from_file(&temp.path().join("config.yml")).unwrap();
        assert_eq!(
            new_cfg.get_alias("test").unwrap_or_default(),
            "example.com/tests/test",
            "the alias should be added to the config file"
        );
    }

    #[tokio::test]
    async fn run_alias_delete() {
        let temp = tempfile::tempdir().unwrap();
        tokio::fs::write(
            temp.path().join("config.yml"),
            r#"
directory: /dev
aliases:
  test: example.com/tests/test
            "#,
        )
        .await
        .unwrap();

        let cfg = Config::from_file(&temp.path().join("config.yml")).unwrap();
        assert_eq!(
            cfg.get_alias("test").unwrap(),
            "example.com/tests/test",
            "the config should have an alias initially"
        );

        let core = CoreBuilder::default()
            .with_config(&cfg)
            .with_mock_output()
            .build();

        let cmd = ConfigCommand {};
        let args = cmd
            .app()
            .get_matches_from(vec!["config", "alias", "-d", "test"]);

        match cmd.run(&core, &args).await {
            Ok(_) => {}
            Err(err) => panic!(err.message()),
        }

        let new_cfg = Config::from_file(&temp.path().join("config.yml")).unwrap();
        assert_eq!(
            new_cfg.get_alias("test").is_none(),
            true,
            "the alias should be removed from the config file"
        );
    }

    #[tokio::test]
    async fn test_alias_completion() {
        let cfg = Config::from_str(&format!(
            r#"
directory: "{}"

aliases:
  test1: example.com/tests/test1
  test2: example.com/tests/test2
"#,
            get_dev_dir().to_str().unwrap().replace("\\", "\\\\")
        ))
        .unwrap();

        test_completions_with_config(&cfg, "gt config alias", "", vec!["test1", "test2"]).await;

        test_completions_with_config(
            &cfg,
            "gt config alias test1",
            "",
            vec!["-d", "github.com/sierrasoftworks/test1"],
        )
        .await;
    }
}
