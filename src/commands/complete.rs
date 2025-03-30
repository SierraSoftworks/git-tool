use super::*;
use clap::Arg;
use tracing_batteries::prelude::*;

pub struct CompleteCommand;

crate::command!(CompleteCommand);

#[async_trait]
impl CommandRunnable for CompleteCommand {
    fn name(&self) -> String {
        String::from("complete")
    }
    fn app(&self) -> clap::Command {
        clap::Command::new(self.name())
            .version("1.0")
            .about("provides command auto-completion")
            .long_about("Provides realtime command and argument auto-completion for Git-Tool when using `git-tool shell-init`.")
            .arg(Arg::new("position")
                    .long("position")
                    .help("The position of the cursor when the completion is requested")
                    .action(clap::ArgAction::Set)
                    .value_parser(clap::value_parser!(usize)))
            .arg(Arg::new("args")
                .help("The parameters being passed to Git-Tool for auto-completion.")
                .index(1))
    }

    #[tracing::instrument(name = "gt complete", err, skip(self, core, matches))]
    async fn run(&self, core: &Core, matches: &ArgMatches) -> Result<i32, core::Error>
where {
        let position: Option<usize> = matches.get_one::<usize>("position").copied();

        let args = matches
            .get_one::<String>("args")
            .map(|s| s.as_str())
            .unwrap_or_default();

        let (cmd, filter) = self
            .extract_command_and_filter(args, position)
            .unwrap_or_default();

        let completer = Completer::new(core, &filter);
        self.offer_completions(core, &cmd, &completer).await;

        Ok(0)
    }

    #[tracing::instrument(
        name = "gt complete -- gt complete",
        skip(self, _core, completer, _matches)
    )]
    async fn complete(&self, _core: &Core, completer: &Completer, _matches: &ArgMatches) {
        completer.offer("--position");
    }
}

impl CompleteCommand {
    fn extract_command_and_filter(
        &self,
        args: &str,
        position: Option<usize>,
    ) -> Option<(String, String)> {
        let mut cmd = args.to_string();

        if cmd.is_empty() {
            return None;
        }

        match position {
            Some(position) if position >= cmd.len() => {
                cmd.extend(std::iter::repeat(' ').take(position - cmd.len()));
            }
            Some(position) if position < cmd.len() => {
                cmd = cmd[..position].into();
            }
            _ => {}
        }

        let mut filter = "".to_string();
        if let Some((last_space_index, _)) = cmd.match_indices(' ').last() {
            filter = cmd[last_space_index + 1..].to_string();
            cmd = cmd[..last_space_index].to_string();
        }

        Some((cmd, filter))
    }

    fn get_responsible_command(&self, args: &str) -> Option<(Command, ArgMatches)> {
        if let Ok(complete_matches) = self.get_completion_matches(args) {
            for cmd in inventory::iter::<Command> {
                if let Some(cmd_matches) = complete_matches.subcommand_matches(&cmd.name()) {
                    return Some((cmd.clone(), cmd_matches.clone()));
                }
            }
        }

        None
    }

    async fn offer_completions(&self, core: &Core, args: &str, completer: &Completer) {
        match self.get_responsible_command(args) {
            Some((cmd, matches)) => {
                cmd.complete(core, completer, &matches).await;
            }
            None => {
                for cmd in inventory::iter::<Command> {
                    completer.offer(cmd.name());
                }
            }
        }
    }

    fn get_completion_matches(&self, args: &str) -> Result<ArgMatches, errors::Error> {
        let true_args = shell_words::split(args)
            .map_err(|e| errors::user_with_internal(
                "Could not parse the arguments you provided.",
                "Please make sure that you are using auto-complete with a valid set of command line arguments.",
                e))?;

        let complete_app = clap::Command::new("Git-Tool")
            .subcommands(inventory::iter::<Command>().map(|x| x.app()));

        complete_app.try_get_matches_from(true_args).map_err(|err| {
            errors::user_with_internal(
                "Failed to parse command line arguments for auto-completion.",
                "Make sure that you are using valid Git-Tool command line arguments and try again.",
                errors::detailed_message(&err.to_string()),
            )
        })
    }
}

#[cfg(test)]
pub mod helpers {
    use super::core::Config;
    use super::*;
    use crate::test::get_dev_dir;

    pub fn test_responsible_command(args: &str, expected: Option<&str>) {
        let cmd = CompleteCommand {};

        let responsible = cmd.get_responsible_command(args).map(|(c, _)| c.name());

        assert_eq!(
            responsible.clone(),
            expected.map(|n| n.to_string()),
            "responsible command [{}] should match [{}]",
            responsible.unwrap_or_else(|| "<None>".to_string()),
            expected.unwrap_or("<None>")
        );
    }

    pub async fn test_completions_with_core(
        core: &Core,
        args: &str,
        filter: &str,
        contains: Vec<&str>,
    ) {
        let cmd = CompleteCommand {};

        let console = crate::console::mock();
        let completer = Completer::new_for(filter, console.clone());
        cmd.offer_completions(core, args, &completer).await;

        let output = console.to_string();
        println!("{}", output);

        let mut offers = std::collections::HashSet::new();

        for offer in output.split_terminator('\n') {
            offers.insert(offer);
        }

        for item in contains {
            assert!(
                offers.contains(item),
                "completion output '{output}' should contain '{item}'"
            );
        }
    }

    pub async fn test_completions_with_config(
        cfg: &Config,
        args: &str,
        filter: &str,
        contains: Vec<&str>,
    ) {
        let core = Core::builder().with_config(cfg.clone()).build();

        test_completions_with_core(&core, args, filter, contains).await;
    }

    pub async fn test_completions(args: &str, filter: &str, contains: Vec<&str>) {
        let config = Config::for_dev_directory(&get_dev_dir());

        test_completions_with_config(&config, args, filter, contains).await;
    }

    pub async fn test_completions2(args: Vec<&str>, contains: Vec<&str>) {
        let console = crate::console::mock();
        let core = Core::builder()
            .with_config_for_dev_directory(get_dev_dir())
            .with_console(console.clone())
            .build();

        let cmd = CompleteCommand {};

        let args = cmd.app().get_matches_from(args);
        cmd.run(&core, &args)
            .await
            .expect("the command should run successfully");

        let output = console.to_string();

        for item in contains {
            assert!(
                output.split('\n').any(|x| x == item),
                "completion output '{output}' should contain '{item}'"
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::helpers::*;
    use super::*;

    #[tokio::test]
    async fn run() {
        let core = Core::builder().with_default_config().build();

        let cmd = CompleteCommand {};
        let args = cmd
            .app()
            .get_matches_from(vec!["gt", "--position", "14", "git-tool apps "]);
        match cmd.run(&core, &args).await {
            Ok(_) => {}
            Err(err) => panic!("{}", err.message()),
        }
    }

    #[test]
    fn extract_command_and_filter() {
        let cmd = CompleteCommand {};

        assert_eq!(cmd.extract_command_and_filter("", None), None);
        assert_eq!(
            cmd.extract_command_and_filter("git-tool complete ", None),
            Some(("git-tool complete".to_string(), "".to_string()))
        );

        assert_eq!(cmd.extract_command_and_filter("", None), None);
        assert_eq!(
            cmd.extract_command_and_filter("git-tool new Si", Some(15)),
            Some(("git-tool new".to_string(), "Si".to_string()))
        );

        assert_eq!(cmd.extract_command_and_filter("", None), None);
        assert_eq!(
            cmd.extract_command_and_filter("git-tool apps ", Some(14)),
            Some(("git-tool apps".to_string(), "".to_string()))
        );

        assert_eq!(
            cmd.extract_command_and_filter("gt o", Some(5)),
            Some(("gt o".to_string(), "".to_string()))
        );
        assert_eq!(
            cmd.extract_command_and_filter("gt o sie", Some(8)),
            Some(("gt o".to_string(), "sie".to_string()))
        );
        assert_eq!(
            cmd.extract_command_and_filter("gt update", Some(10)),
            Some(("gt update".to_string(), "".to_string()))
        );
    }

    #[test]
    fn get_completion_matches() {
        let cmd = CompleteCommand {};

        assert_eq!(
            cmd.get_completion_matches("git-tool new")
                .unwrap()
                .subcommand_name(),
            Some("new")
        );
    }

    #[test]
    fn get_responsible_commands() {
        test_responsible_command("", None);
        test_responsible_command("git-tool", None);
        test_responsible_command("git-tool notarealcommand", None);
        test_responsible_command("git-tool new", Some("new"));
        test_responsible_command("gt new", Some("new"));
        test_responsible_command("gt new ", Some("new"));
        test_responsible_command("gt apps ", Some("apps"));
    }

    #[tokio::test]
    async fn offer_completions_none() {
        test_completions(
            "gt",
            "",
            vec![
                "apps", "config", "ignore", "info", "list", "new", "open", "scratch", "services",
            ],
        )
        .await;
    }

    #[tokio::test]
    async fn offer_completions_with_no_options() {
        test_completions("gt apps", "", vec![]).await;
    }

    #[tokio::test]
    async fn offer_completions_with_options() {
        test_completions(
            "gt new",
            "Sierra",
            vec!["gh:sierrasoftworks/", "ado:sierrasoftworks/opensource/"],
        )
        .await;

        test_completions(
            "gt open",
            "",
            vec![
                "shell",
                "gh:sierrasoftworks/test1",
                "ado:sierrasoftworks/opensource/test2",
            ],
        )
        .await;

        test_completions(
            "gt info",
            "",
            vec![
                "gh:sierrasoftworks/test1",
                "ado:sierrasoftworks/opensource/test2",
            ],
        )
        .await;

        test_completions(
            "gt scratch",
            "",
            vec!["shell", "2019w15", "2019w16", "2019w27"],
        )
        .await;
    }

    #[tokio::test]
    async fn test_completions_with_position() {
        test_completions2(
            vec!["complete", "git-tool open "],
            vec!["gh:sierrasoftworks/test1"],
        )
        .await;

        test_completions2(
            vec!["complete", "--position", "14", "git-tool open "],
            vec!["gh:sierrasoftworks/test2"],
        )
        .await;
    }
}
