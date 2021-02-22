use crate::core::features;

use super::async_trait;
use super::Command;
use super::*;
use clap::{App, Arg, ArgMatches};
use online::registry::Registry;

pub struct ConfigCommand {}

impl Command for ConfigCommand {
    fn name(&self) -> String {
        String::from("config")
    }

    fn app<'a>(&self) -> App<'a> {
        App::new(self.name().as_str())
            .version("1.0")
            .about("manage your Git-Tool configuration file")
            .long_about("This tool allows you to easily make changes to your Git-Tool config file.")

            .subcommand(App::new("list")
                .version("1.0")
                .visible_alias("ls")
                .about("list available config templates")
                .long_about("Gets the list of config templates which are available through the Git-Tool registry."))

            .subcommand(App::new("add")
                .version("1.0")
                .about("adds a configuration template to your current config file")
                .long_about("Adds a configuration template from the Git-Tool online registry to your config file.")
                .arg(Arg::new("id")
                    .index(1)
                    .about("the id of the configuration template you want to add")
                    .required(true))
                .arg(Arg::new("force")
                    .long("force")
                    .short('f')
                    .about("overwrites any existing entries with those from the template.")))

            .subcommand(App::new("alias")
                .version("1.0")
                .about("manage aliases for your repositories")
                .long_about("Set or remove aliases for your repositories within your config file.")
                .arg(Arg::new("delete")
                    .short('d')
                    .long("delete")
                    .about("delete the alias from your config file"))
                .arg(Arg::new("alias")
                    .about("the name of the alias to manage")
                    .index(1))
                .arg(Arg::new("repo")
                    .about("the fully qualified repository name")
                    .index(2)))

            .subcommand(App::new("feature")
                .version("1.0")
                .about("manage feature flags for Git-Tool")
                .long_about("Set feature flags for Git-Tool within your config file.")
                .arg(Arg::new("flag")
                    .about("the name of the feature flag to set")
                    .index(1))
                .arg(Arg::new("enable")
                    .about("whether the feature flag should be enabled or not (true/false)")
                    .index(2)))
    }
}

#[async_trait]
impl CommandRunnable for ConfigCommand {
    async fn run(&self, core: &Core, matches: &ArgMatches) -> Result<i32, errors::Error> {
        let mut output = core.output();

        match matches.subcommand() {
            Some(("list", _args)) => {
                let registry = crate::online::GitHubRegistry;

                let entries = registry.get_entries(core).await?;
                for entry in entries {
                    writeln!(output, "{}", entry)?;
                }
            }
            Some(("add", args)) => {
                let id = args.value_of("id").ok_or(errors::user(
                    "You have not provided an ID for the config template you wish to add.",
                    "",
                ))?;

                let registry = crate::online::GitHubRegistry;
                let entry = registry.get_entry(core, id).await?;

                writeln!(output, "Applying {}", entry.name)?;
                writeln!(output, "> {}", entry.description)?;

                let mut cfg = core.config().clone();
                for ec in entry.configs {
                    if ec.is_compatible() {
                        cfg = cfg.apply_template(ec, args.is_present("force"))?;
                    }
                }

                match cfg.get_config_file() {
                    Some(path) => {
                        tokio::fs::write(&path, cfg.to_string()?).await.map_err(|err| errors::user_with_internal(
                            &format!("Could not write your updated config to the config file '{}' due to an OS-level error.", path.display()),
                            "Make sure that Git-Tool has permission to write to your config file and then try again.",
                            err
                        ))?;
                    }
                    None => {
                        writeln!(output, "{}", cfg.to_string()?)?;
                    }
                }
            }
            Some(("alias", args)) => match args.value_of("alias") {
                Some(alias) => {
                    if args.is_present("delete") {
                        let mut cfg = core.config().clone();
                        cfg.remove_alias(alias);

                        match cfg.get_config_file() {
                            Some(path) => {
                                tokio::fs::write(&path, cfg.to_string()?).await.map_err(|err| errors::user_with_internal(
                                    &format!("Could not write your updated config to the config file '{}' due to an OS-level error.", path.display()),
                                    "Make sure that Git-Tool has permission to write to your config file and then try again.",
                                    err
                                ))?;
                            }
                            None => {
                                writeln!(output, "{}", cfg.to_string()?)?;
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
                                    tokio::fs::write(&path, cfg.to_string()?).await.map_err(|err| errors::user_with_internal(
                                        &format!("Could not write your updated config to the config file '{}' due to an OS-level error.", path.display()),
                                        "Make sure that Git-Tool has permission to write to your config file and then try again.",
                                        err
                                    ))?;
                                }
                                None => {
                                    writeln!(output, "{}", cfg.to_string()?)?;
                                }
                            }
                        }
                        None => match core.config().get_alias(alias) {
                            Some(repo) => {
                                writeln!(output, "{} = {}", alias, repo)?;
                            }
                            None => {
                                writeln!(output, "No alias exists with the name '{}'", alias)?;
                            }
                        },
                    }
                }
                None => {
                    for (alias, repo) in core.config().get_aliases() {
                        writeln!(output, "{} = {}", alias, repo)?;
                    }
                }
            },
            Some(("feature", args)) => match args.value_of("flag") {
                Some(flag) => match args.value_of("enable") {
                    Some(value) if value == "true" || value == "false" => {
                        let cfg = core.config().with_feature_flag(flag, value == "true");

                        match cfg.get_config_file() {
                            Some(path) => {
                                tokio::fs::write(&path, cfg.to_string()?).await?;
                            }
                            None => {
                                writeln!(output, "{}", cfg.to_string()?)?;
                            }
                        }
                    }
                    Some(invalid) => {
                        writeln!(output, "Cannot set the feature flag {} to {} because only 'true' and 'false' are valid settings.", flag, invalid)?;
                        return Ok(1);
                    }
                    None => {
                        writeln!(
                            output,
                            "{} = {}",
                            flag,
                            core.config().get_features().has(flag)
                        )?;
                    }
                },
                None => {
                    for &feature in features::ALL.iter() {
                        writeln!(
                            output,
                            "{} = {}",
                            feature,
                            core.config().get_features().has(feature)
                        )?;
                    }
                }
            },
            _ => {
                writeln!(output, "{}", core.config().to_string()?)?;
            }
        }

        Ok(0)
    }

    async fn complete(&self, core: &Core, completer: &Completer, matches: &ArgMatches) {
        match matches.subcommand() {
            Some(("list", _)) => {}
            Some(("add", _)) => {
                let registry = online::GitHubRegistry;
                match registry.get_entries(core).await {
                    Ok(entries) => {
                        completer.offer_many(entries);
                    }
                    _ => {}
                }
            }
            Some(("alias", args)) => {
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
            Some(("feature", args)) => {
                if !args.is_present("flag") {
                    completer.offer_many(features::ALL.iter().map(|&v| v));
                } else {
                    completer.offer("true");
                    completer.offer("false");
                }
            }
            _ => {
                completer.offer_many(vec!["list", "add", "alias", "feature"]);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::core::Config;
    use super::*;
    use crate::test::get_dev_dir;
    use clap::ArgMatches;
    use complete::helpers::test_completions_with_config;

    #[tokio::test]
    async fn run() {
        let args = ArgMatches::default();
        let cfg = Config::from_str("directory: /dev").unwrap();
        let core = Core::builder().with_config(&cfg).build();

        let output = crate::console::output::mock();

        let cmd = ConfigCommand {};

        match cmd.run(&core, &args).await {
            Ok(_) => {}
            Err(err) => panic!("{}", err.message()),
        }

        assert!(
            output
                .to_string()
                .contains(&core.config().to_string().unwrap()),
            "the output should contain the config"
        );
    }

    #[tokio::test]
    async fn run_list() {
        let cfg = Config::from_str("directory: /dev").unwrap();
        let core = Core::builder().with_config(&cfg).build();

        let output = crate::console::output::mock();

        let cmd = ConfigCommand {};
        let args = cmd.app().get_matches_from(vec!["config", "list"]);

        match cmd.run(&core, &args).await {
            Ok(_) => {}
            Err(err) => panic!("{}", err.message()),
        }

        println!("{}", output.to_string());
        assert!(
            output.to_string().contains("apps/bash\n"),
            "the output should contain some apps"
        );
        assert!(
            output.to_string().contains("services/github\n"),
            "the output should contain some services"
        );
    }

    #[tokio::test]
    async fn run_add_no_file() {
        let cfg = Config::from_str("directory: /dev").unwrap();
        let core = Core::builder().with_config(&cfg).build();

        let output = crate::console::output::mock();

        let cmd = ConfigCommand {};
        let args = cmd
            .app()
            .get_matches_from(vec!["config", "add", "apps/bash"]);

        match cmd.run(&core, &args).await {
            Ok(_) => {}
            Err(err) => panic!("{}", err.message()),
        }

        println!("{}", output.to_string());
        assert!(
            output.to_string().contains("name: bash\n"),
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
        let core = Core::builder().with_config(&cfg).build();

        let output = crate::console::output::mock();

        let cmd = ConfigCommand {};
        let args = cmd
            .app()
            .get_matches_from(vec!["config", "add", "apps/bash"]);

        match cmd.run(&core, &args).await {
            Ok(_) => {}
            Err(err) => panic!("{}", err.message()),
        }

        assert!(
            output.to_string().contains("Applying Bash\n> "),
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
        let core = Core::builder().with_config(&cfg).build();

        let output = crate::console::output::mock();

        let cmd = ConfigCommand {};
        let args = cmd.app().get_matches_from(vec!["config", "alias"]);

        match cmd.run(&core, &args).await {
            Ok(_) => {}
            Err(err) => panic!("{}", err.message()),
        }

        println!("{}", output.to_string());
        assert!(
            output
                .to_string()
                .contains("test1 = example.com/tests/test1\n"),
            "the output should contain the aliases"
        );
        assert!(
            output
                .to_string()
                .contains("test2 = example.com/tests/test2\n"),
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
        let core = Core::builder().with_config(&cfg).build();

        let output = crate::console::output::mock();

        let cmd = ConfigCommand {};
        let args = cmd.app().get_matches_from(vec!["config", "alias", "test1"]);

        match cmd.run(&core, &args).await {
            Ok(_) => {}
            Err(err) => panic!("{}", err.message()),
        }

        println!("{}", output.to_string());
        assert!(
            output
                .to_string()
                .contains("test1 = example.com/tests/test1\n"),
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

        let core = Core::builder().with_config(&cfg).build();

        crate::console::output::mock();

        let cmd = ConfigCommand {};
        let args =
            cmd.app()
                .get_matches_from(vec!["config", "alias", "test", "example.com/tests/test"]);

        match cmd.run(&core, &args).await {
            Ok(_) => {}
            Err(err) => panic!("{}", err.message()),
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

        let core = Core::builder().with_config(&cfg).build();

        crate::console::output::mock();

        let cmd = ConfigCommand {};
        let args = cmd
            .app()
            .get_matches_from(vec!["config", "alias", "-d", "test"]);

        match cmd.run(&core, &args).await {
            Ok(_) => {}
            Err(err) => panic!("{}", err.message()),
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

    #[tokio::test]
    async fn run_feature_set() {
        let temp = tempfile::tempdir().unwrap();
        tokio::fs::write(
            temp.path().join("config.yml"),
            r#"
directory: /dev
features:
    http_transport: true
            "#,
        )
        .await
        .unwrap();

        let cfg = Config::from_file(&temp.path().join("config.yml")).unwrap();
        assert_eq!(
            cfg.get_features().has("http_transport"),
            true,
            "the config should have the feature enabled initially"
        );

        let core = Core::builder().with_config(&cfg).build();

        crate::console::output::mock();

        let cmd = ConfigCommand {};
        let args = cmd
            .app()
            .get_matches_from(vec!["config", "feature", "http_transport", "false"]);

        match cmd.run(&core, &args).await {
            Ok(_) => {}
            Err(err) => panic!("{}", err.message()),
        }

        let new_cfg = Config::from_file(&temp.path().join("config.yml")).unwrap();
        assert_eq!(
            new_cfg.get_features().has("http_transport"),
            false,
            "the feature should be set to false in the config file"
        );
    }

    #[tokio::test]
    async fn test_feature_completion() {
        let cfg = Config::from_str(&format!(
            r#"
directory: "{}"

features:
    http_transport: true
"#,
            get_dev_dir().to_str().unwrap().replace("\\", "\\\\")
        ))
        .unwrap();

        test_completions_with_config(
            &cfg,
            "gt config feature",
            "",
            vec!["http_transport", "create_remote", "create_remote_private"],
        )
        .await;

        test_completions_with_config(
            &cfg,
            "gt config feature http_transport",
            "",
            vec!["true", "false"],
        )
        .await;
    }
}
