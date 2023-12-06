use crate::core::features;

use super::async_trait;
use super::CommandRunnable;
use super::*;
use clap::{Arg, ArgMatches};
use online::registry::Registry;

pub struct ConfigCommand;

crate::command!(ConfigCommand);

#[async_trait]
impl CommandRunnable for ConfigCommand {
    fn name(&self) -> String {
        String::from("config")
    }

    fn app(&self) -> clap::Command {
        clap::Command::new(self.name())
            .version("1.0")
            .about("manage your Git-Tool configuration file")
            .long_about("This tool allows you to easily make changes to your Git-Tool config file.")

            .subcommand(clap::Command::new("list")
                .version("1.0")
                .visible_alias("ls")
                .about("list available config templates")
                .long_about("Gets the list of config templates which are available through the Git-Tool registry."))

            .subcommand(clap::Command::new("add")
                .version("1.0")
                .about("adds a configuration template to your current config file")
                .long_about("Adds a configuration template from the Git-Tool online registry to your config file.")
                .arg(Arg::new("id")
                    .index(1)
                    .help("the id of the configuration template you want to add")
                    .required(true))
                .arg(Arg::new("as")
                    .long("name")
                    .short('n')
                    .help("the name you would like to associated with this entry")
                    .action(clap::ArgAction::Set))
                .arg(Arg::new("force")
                    .long("force")
                    .short('f')
                    .help("overwrites any existing entries with those from the template.")
                    .action(clap::ArgAction::SetTrue)))

            .subcommand(clap::Command::new("alias")
                .version("1.0")
                .about("manage aliases for your repositories")
                .long_about("Set or remove aliases for your repositories within your config file.")
                .arg(Arg::new("delete")
                    .short('d')
                    .long("delete")
                    .help("delete the alias from your config file")
                    .action(clap::ArgAction::SetTrue))
                .arg(Arg::new("alias")
                    .help("the name of the alias to manage")
                    .index(1))
                .arg(Arg::new("repo")
                    .help("the fully qualified repository name")
                    .index(2)))

            .subcommand(clap::Command::new("feature")
                .version("1.0")
                .about("manage feature flags for Git-Tool")
                .long_about("Set feature flags for Git-Tool within your config file.")
                .arg(Arg::new("flag")
                    .help("the name of the feature flag to set")
                    .index(1))
                .arg(Arg::new("enable")
                    .help("whether the feature flag should be enabled or not (true/false)")
                    .index(2)))

            .subcommand(clap::Command::new("path")
                .version("1.0")
                .about("manage the path used to store your repositories and scratchpads")
                .long_about("Set the folder used to store the repositories managed by Git-Tool, or your scratchpads.")
                .arg(Arg::new("path")
                    .help("the path to use for storing repositories and scratchpads")
                    .index(1))
                .arg(Arg::new("scratch")
                    .long("scratch")
                    .short('s')
                    .help("configure the scratchpads path instead of the repositories path")
                    .action(clap::ArgAction::SetTrue)))
    }

    #[tracing::instrument(name = "gt config", err, skip(self, core, matches))]
    async fn run(&self, core: &Core, matches: &ArgMatches) -> Result<i32, errors::Error> {
        match matches.subcommand() {
            Some(("list", _args)) => {
                let registry = crate::online::GitHubRegistry;

                let entries = registry.get_entries(core).await?;
                let mut output = core.output();
                for entry in entries {
                    writeln!(output, "{entry}")?;
                }
            }
            Some(("add", args)) => {
                let id = args.get_one::<String>("id").ok_or_else(|| {
                    errors::user(
                        "You have not provided an ID for the config template you wish to add.",
                        "",
                    )
                })?;

                let registry = crate::online::GitHubRegistry;
                let entry = registry.get_entry(core, id).await?;

                writeln!(core.output(), "Applying {}", entry.name)?;
                writeln!(core.output(), "> {}", entry.description)?;

                let mut cfg = core.config().clone();
                for ec in entry.configs {
                    if ec.is_compatible() {
                        let ec = if let Some(name) = args.get_one::<String>("as") {
                            ec.with_name(name)
                        } else {
                            ec
                        };

                        cfg = cfg.apply_template(ec, args.get_flag("force"))?;
                    }
                }

                match cfg.get_config_file() {
                    Some(path) => {
                        cfg.save(&path).await?;
                    }
                    None => {
                        writeln!(core.output(), "{}", cfg.to_string()?)?;
                    }
                }
            }
            Some(("alias", args)) => match args.get_one::<String>("alias") {
                Some(alias) => {
                    if args.get_flag("delete") {
                        let mut cfg = core.config().clone();
                        cfg.remove_alias(alias);

                        match cfg.get_config_file() {
                            Some(path) => {
                                cfg.save(&path).await?;
                            }
                            None => {
                                writeln!(core.output(), "{}", cfg.to_string()?)?;
                            }
                        }

                        return Ok(0);
                    }

                    match args.get_one::<String>("repo") {
                        Some(repo) => {
                            let mut cfg = core.config().clone();
                            cfg.add_alias(alias, repo);

                            match cfg.get_config_file() {
                                Some(path) => {
                                    cfg.save(&path).await?;
                                }
                                None => {
                                    writeln!(core.output(), "{}", cfg.to_string()?)?;
                                }
                            }
                        }
                        None => match core.config().get_alias(alias) {
                            Some(repo) => {
                                writeln!(core.output(), "{alias} = {repo}")?;
                            }
                            None => {
                                writeln!(core.output(), "No alias exists with the name '{alias}'")?;
                            }
                        },
                    }
                }
                None => {
                    let mut output = core.output();
                    for (alias, repo) in core.config().get_aliases() {
                        writeln!(output, "{alias} = {repo}")?;
                    }
                }
            },
            Some(("feature", args)) => match args.get_one::<String>("flag") {
                Some(flag) => match args.get_one::<String>("enable") {
                    Some(value) if value == "true" || value == "false" => {
                        let cfg = core.config().with_feature_flag(flag, value == "true");

                        match cfg.get_config_file() {
                            Some(path) => {
                                cfg.save(&path).await?;
                            }
                            None => {
                                writeln!(core.output(), "{}", cfg.to_string()?)?;
                            }
                        }
                    }
                    Some(invalid) => {
                        writeln!(core.output(), "Cannot set the feature flag '{flag}' to '{invalid}' because only 'true' and 'false' are valid settings.")?;
                        return Ok(1);
                    }
                    None => {
                        writeln!(
                            core.output(),
                            "{} = {}",
                            flag,
                            core.config().get_features().has(flag)
                        )?;
                    }
                },
                None => {
                    let mut output = core.output();
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
            Some(("path", args)) if args.get_flag("scratch") => {
                match args.get_one::<String>("path") {
                    Some(path) => {
                        let cfg = core.config().with_scratch_directory(path);

                        match cfg.get_config_file() {
                            Some(path) => {
                                cfg.save(&path).await?;
                            }
                            None => {
                                writeln!(core.output(), "{}", cfg.to_string()?)?;
                            }
                        }
                    }

                    None => {
                        writeln!(
                            core.output(),
                            "{}",
                            core.config().get_scratch_directory().display()
                        )?;
                    }
                }
            }
            Some(("path", args)) => match args.get_one::<String>("path") {
                Some(path) => {
                    let cfg = core.config().with_dev_directory(path);

                    match cfg.get_config_file() {
                        Some(path) => {
                            cfg.save(&path).await?;
                        }
                        None => {
                            writeln!(core.output(), "{}", cfg.to_string()?)?;
                        }
                    }
                }
                None => {
                    writeln!(
                        core.output(),
                        "{}",
                        core.config().get_dev_directory().display()
                    )?;
                }
            },
            _ => {
                writeln!(core.output(), "{}", core.config().to_string()?)?;
            }
        }

        Ok(0)
    }

    #[tracing::instrument(
        name = "gt complete -- gt config",
        skip(self, core, completer, matches)
    )]
    async fn complete(&self, core: &Core, completer: &Completer, matches: &ArgMatches) {
        match matches.subcommand() {
            Some(("list", _)) => {}
            Some(("add", _)) => {
                let registry = online::GitHubRegistry;
                if let Ok(entries) = registry.get_entries(core).await {
                    completer.offer_many(entries);
                }
            }
            Some(("alias", args)) => {
                if !args.contains_id("alias") {
                    completer.offer_many(core.config().get_aliases().map(|(a, _)| a));
                } else if !args.get_flag("delete") && !args.contains_id("repo") {
                    completer.offer("-d");
                    if let Ok(repos) = core.resolver().get_repos() {
                        completer.offer_many(
                            repos
                                .iter()
                                .map(|r| format!("{}:{}", &r.service, r.get_full_name())),
                        );
                    }
                }
            }
            Some(("feature", args)) => {
                if !args.contains_id("flag") {
                    completer.offer_many(features::ALL.iter().copied());
                } else {
                    completer.offer("true");
                    completer.offer("false");
                }
            }
            Some(("path", args)) => {
                if !args.get_flag("scratch") {
                    completer.offer("--scratch");
                }
            }
            _ => {
                completer.offer_many(vec!["list", "add", "alias", "feature", "path"]);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::core::Config;
    use super::*;
    use crate::test::get_dev_dir;
    use clap::ArgMatches;
    use complete::helpers::test_completions_with_config;

    #[tokio::test]
    async fn run() {
        let args = ArgMatches::default();
        let cfg = Config::from_str("directory: /dev").unwrap();
        let console = crate::console::mock();
        let core = Core::builder()
            .with_config(cfg)
            .with_console(console.clone())
            .build();

        let cmd = ConfigCommand {};

        match cmd.run(&core, &args).await {
            Ok(_) => {}
            Err(err) => panic!("{}", err.message()),
        }

        assert!(
            console
                .to_string()
                .contains(&core.config().to_string().unwrap()),
            "the output should contain the config"
        );
    }

    #[tokio::test]
    #[cfg_attr(feature = "pure-tests", ignore)]
    async fn run_list() {
        let cfg = Config::from_str("directory: /dev").unwrap();
        let console = crate::console::mock();
        let core = Core::builder()
            .with_config(cfg)
            .with_console(console.clone())
            .build();

        let cmd = ConfigCommand {};
        let args = cmd.app().get_matches_from(vec!["config", "list"]);

        match cmd.run(&core, &args).await {
            Ok(_) => {}
            Err(err) => panic!("{}", err.message()),
        }

        println!("{}", console.to_string());
        assert!(
            console.to_string().contains("apps/bash\n"),
            "the output should contain some apps"
        );
        assert!(
            console.to_string().contains("services/github-ssh\n"),
            "the output should contain some services"
        );
    }

    #[tokio::test]
    #[cfg_attr(feature = "pure-tests", ignore)]
    async fn run_add_no_file() {
        let cfg = Config::from_str("directory: /dev").unwrap();
        let console = crate::console::mock();
        let core = Core::builder()
            .with_config(cfg)
            .with_console(console.clone())
            .build();

        let cmd = ConfigCommand {};
        let args = cmd
            .app()
            .get_matches_from(vec!["config", "add", "apps/bash"]);

        match cmd.run(&core, &args).await {
            Ok(_) => {}
            Err(err) => panic!("{}", err.message()),
        }

        println!("{}", console.to_string());
        assert!(
            console.to_string().contains("Applying Bash\n"),
            "the output should explain which config is being applied"
        );
    }

    #[tokio::test]
    #[cfg_attr(feature = "pure-tests", ignore)]
    async fn run_add_with_file() {
        let temp = tempfile::tempdir().unwrap();
        tokio::fs::write(
            temp.path().join("config.yml"),
            Config::default().to_string().unwrap(),
        )
        .await
        .unwrap();

        let console = crate::console::mock();
        let core = Core::builder()
            .with_config_file(temp.path().join("config.yml"))
            .expect("the config should be loaded")
            .with_console(console.clone())
            .build();

        let cmd = ConfigCommand {};
        let args = cmd
            .app()
            .get_matches_from(vec!["config", "add", "apps/bash"]);

        match cmd.run(&core, &args).await {
            Ok(_) => {}
            Err(err) => panic!("{}", err.message()),
        }

        assert!(
            console.to_string().contains("Applying Bash\n> "),
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

        let console = crate::console::mock();
        let core = Core::builder()
            .with_config(cfg)
            .with_console(console.clone())
            .build();

        let cmd = ConfigCommand {};
        let args = cmd.app().get_matches_from(vec!["config", "alias"]);

        match cmd.run(&core, &args).await {
            Ok(_) => {}
            Err(err) => panic!("{}", err.message()),
        }

        println!("{}", console.to_string());
        assert!(
            console
                .to_string()
                .contains("test1 = example.com/tests/test1\n"),
            "the output should contain the aliases"
        );
        assert!(
            console
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

        let console = crate::console::mock();
        let core = Core::builder()
            .with_config(cfg)
            .with_console(console.clone())
            .build();

        let cmd = ConfigCommand {};
        let args = cmd.app().get_matches_from(vec!["config", "alias", "test1"]);

        match cmd.run(&core, &args).await {
            Ok(_) => {}
            Err(err) => panic!("{}", err.message()),
        }

        println!("{}", console.to_string());
        assert!(
            console
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

        let core = Core::builder()
            .with_config_file(temp.path().join("config.yml"))
            .expect("the config should be loaded")
            .with_null_console()
            .build();

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

        let core = Core::builder()
            .with_config_file(temp.path().join("config.yml"))
            .expect("the config should be loaded")
            .with_null_console()
            .build();

        assert_eq!(
            core.config().get_alias("test").unwrap(),
            "example.com/tests/test",
            "the config should have an alias initially"
        );

        let cmd = ConfigCommand {};
        let args = cmd
            .app()
            .get_matches_from(vec!["config", "alias", "-d", "test"]);

        match cmd.run(&core, &args).await {
            Ok(_) => {}
            Err(err) => panic!("{}", err.message()),
        }

        let new_cfg = Config::from_file(&temp.path().join("config.yml")).unwrap();
        assert!(
            new_cfg.get_alias("test").is_none(),
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
            get_dev_dir().to_str().unwrap().replace('\\', "\\\\")
        ))
        .unwrap();

        test_completions_with_config(&cfg, "gt config alias", "", vec!["test1", "test2"]).await;

        test_completions_with_config(
            &cfg,
            "gt config alias test1",
            "",
            vec!["-d", "gh:sierrasoftworks/test1"],
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

        let core = Core::builder()
            .with_config_file(temp.path().join("config.yml"))
            .expect("the config should be loaded")
            .with_null_console()
            .build();

        assert!(
            core.config().get_features().has("http_transport"),
            "the config should have the feature enabled initially"
        );

        let cmd = ConfigCommand {};
        let args = cmd
            .app()
            .get_matches_from(vec!["config", "feature", "http_transport", "false"]);

        match cmd.run(&core, &args).await {
            Ok(_) => {}
            Err(err) => panic!("{}", err.message()),
        }

        let new_cfg = Config::from_file(&temp.path().join("config.yml")).unwrap();
        assert!(
            !new_cfg.get_features().has("http_transport"),
            "the feature should be set to false in the config file"
        );
    }

    #[tokio::test]
    async fn test_feature_completion() {
        let cfg = Config::from_str(&format!(
            r#"
directory: "{}"

features:
    create_remote: true
    telemetry: false
"#,
            get_dev_dir().to_str().unwrap().replace('\\', "\\\\")
        ))
        .unwrap();

        test_completions_with_config(
            &cfg,
            "gt config feature",
            "",
            vec!["create_remote", "create_remote_private", "telemetry"],
        )
        .await;

        test_completions_with_config(
            &cfg,
            "gt config feature create_remote",
            "",
            vec!["true", "false"],
        )
        .await;
    }

    #[tokio::test]
    async fn run_paths() {
        let temp = tempfile::tempdir().unwrap();
        tokio::fs::write(
            temp.path().join("config.yml"),
            r#"
directory: /dev
scratchpads: /scratch
            "#,
        )
        .await
        .unwrap();

        let cfg = Config::from_file(&temp.path().join("config.yml")).unwrap();

        let core = Core::builder().with_config(cfg).with_null_console().build();

        let cmd = ConfigCommand {};

        // Update the dev path
        let args = cmd.app().get_matches_from(vec!["config", "path", "/devel"]);
        match cmd.run(&core, &args).await {
            Ok(_) => {}
            Err(err) => panic!("{}", err.message()),
        }

        let new_cfg = Config::from_file(&temp.path().join("config.yml")).unwrap();
        assert_eq!(new_cfg.get_dev_directory(), PathBuf::from("/devel"));
        assert_eq!(new_cfg.get_scratch_directory(), PathBuf::from("/scratch"));

        // Update the scratch path
        let args =
            cmd.app()
                .get_matches_from(vec!["config", "path", "--scratch", "/devel/scratch"]);
        match cmd.run(&core, &args).await {
            Ok(_) => {}
            Err(err) => panic!("{}", err.message()),
        }

        let new_cfg = Config::from_file(&temp.path().join("config.yml")).unwrap();
        assert_eq!(new_cfg.get_dev_directory(), PathBuf::from("/dev"));
        assert_eq!(
            new_cfg.get_scratch_directory(),
            PathBuf::from("/devel/scratch")
        );
    }

    #[tokio::test]
    async fn test_path_completion() {
        let cfg = Config::from_str(&format!(
            r#"
directory: "{}"
"#,
            get_dev_dir().to_str().unwrap().replace('\\', "\\\\")
        ))
        .unwrap();

        test_completions_with_config(&cfg, "gt config path", "", vec!["--scratch"]).await;

        test_completions_with_config(&cfg, "gt config", "", vec!["path"]).await;
    }
}
