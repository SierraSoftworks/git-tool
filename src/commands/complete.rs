use super::*;
use clap::{Arg, SubCommand};

pub struct CompleteCommand {}

impl Command for CompleteCommand {
    fn name(&self) -> String {
        String::from("complete")
    }
    fn app<'a, 'b>(&self) -> clap::App<'a, 'b> {
        SubCommand::with_name(&self.name())
            .version("1.0")
            .about("provides command auto-completion")
            .after_help("Provides realtime command and argument auto-completion for Git-Tool when using `git-tool shell-init`.")
            .arg(Arg::with_name("position")
                    .long("position")
                    .help("The position of the cursor when the completion is requested")
                    .takes_value(true)
                    .default_value("-1"))
            .arg(Arg::with_name("args")
                .help("The parameters being passed to Git-Tool for auto-completion.")
                .index(1))
    }
}

#[async_trait]
impl<F: FileSource, L: Launcher, R: Resolver> CommandRunnable<F, L, R> for CompleteCommand {
    async fn run<'a>(
        &self,
        core: &crate::core::Core<F, L, R>,
        matches: &clap::ArgMatches<'a>,
    ) -> Result<i32, crate::core::Error>
    where
        F: FileSource,
        L: Launcher,
        R: Resolver,
    {
        let position: Option<usize> = matches
            .value_of("position")
            .map(|v| v.parse().unwrap_or_default());

        let args = matches.value_of("args").unwrap_or_default();

        let commands = super::commands::<F, L, R>();
        let (cmd, filter) = self
            .extract_command_and_filter(args, position)
            .unwrap_or_default();

        let completer = Completer::new(&filter);
        self.offer_completions(core, &commands, &cmd, &completer)
            .await;

        Ok(0)
    }

    async fn complete<'a>(
        &self,
        _core: &Core<F, L, R>,
        completer: &Completer,
        _matches: &ArgMatches<'a>,
    ) {
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

        if cmd == "" {
            return None;
        }

        match position {
            Some(position) if position < cmd.len() => {
                cmd = cmd[..position].into();
            }
            _ => {}
        }

        let mut filter = "".to_string();
        match cmd.match_indices(" ").last() {
            Some((last_space_index, _)) => {
                filter = cmd[last_space_index + 1..].to_string();
                cmd = cmd[..last_space_index].to_string();
            }
            _ => {}
        }

        Some((cmd, filter))
    }

    fn get_responsible_command<F: FileSource, L: Launcher, R: Resolver>(
        &self,
        commands: &Vec<Arc<dyn CommandRunnable<F, L, R>>>,
        args: &str,
    ) -> Option<(Arc<dyn CommandRunnable<F, L, R>>, ArgMatches)> {
        match self.get_completion_matches(commands, args) {
            Ok(complete_matches) => {
                for cmd in commands.iter() {
                    if let Some(cmd_matches) = complete_matches.subcommand_matches(cmd.name()) {
                        return Some((cmd.clone(), cmd_matches.clone()));
                    }
                }
            }
            _ => {}
        }

        None
    }

    async fn offer_completions<F: FileSource, L: Launcher, R: Resolver>(
        &self,
        core: &Core<F, L, R>,
        commands: &Vec<Arc<dyn CommandRunnable<F, L, R>>>,
        args: &str,
        completer: &Completer,
    ) {
        match self.get_responsible_command(commands, args) {
            Some((cmd, matches)) => {
                cmd.complete(core, &completer, &matches).await;
            }
            None => {
                for cmd in commands.iter() {
                    completer.offer(&cmd.name());
                }
            }
        }
    }

    fn get_completion_matches<F: FileSource, L: Launcher, R: Resolver>(
        &self,
        commands: &Vec<Arc<dyn CommandRunnable<F, L, R>>>,
        args: &str,
    ) -> Result<ArgMatches, errors::Error> {
        let true_args = shell_words::split(args)
            .map_err(|e| errors::user_with_internal(
                "Could not parse the arguments you provided.",
                "Please make sure that you are using auto-complete with a valid set of command line arguments.",
                e))?;

        let complete_app = App::new("Git-Tool").subcommands(commands.iter().map(|x| x.app()));

        complete_app
            .get_matches_from_safe(true_args)
            .map_err(|err| {
                errors::user_with_internal(
                "Failed to parse command line arguments for auto-completion.", 
                "Make sure that you are using valid Git-Tool command line arguments and try again.",
                err)
            })
    }
}

#[cfg(test)]
mod tests {
    use super::core::Config;
    use super::*;
    use crate::test::get_dev_dir;
    use std::sync::Mutex;

    #[tokio::test]
    async fn run() {
        let cfg = Config::default();
        let core = Core::builder().with_config(&cfg).build();

        let cmd = CompleteCommand {};
        let args = cmd
            .app()
            .get_matches_from(vec!["gt", "--position", "14", "git-tool apps "]);
        match cmd.run(&core, &args).await {
            Ok(_) => {}
            Err(err) => panic!(err.message()),
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
    }

    #[test]
    fn get_completion_matches() {
        let cmd = CompleteCommand {};

        let cmds = default_commands();

        assert_eq!(
            cmd.get_completion_matches(&cmds, "git-tool new")
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
            vec![
                "github.com/sierrasoftworks/",
                "dev.azure.com/sierrasoftworks/opensource/",
            ],
        )
        .await;

        test_completions(
            "gt open",
            "",
            vec![
                "shell",
                "github.com/sierrasoftworks/test1",
                "dev.azure.com/sierrasoftworks/opensource/test2",
            ],
        )
        .await;

        test_completions(
            "gt info",
            "",
            vec![
                "github.com/sierrasoftworks/test1",
                "dev.azure.com/sierrasoftworks/opensource/test2",
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

    fn test_responsible_command(args: &str, expected: Option<&str>) {
        let cmd = CompleteCommand {};
        let cmds = default_commands();

        let responsible = cmd
            .get_responsible_command(&cmds, args)
            .map(|(c, _)| c.name());

        assert_eq!(
            responsible.clone(),
            expected.map(|n| n.to_string()),
            "responsible command [{}] should match [{}]",
            responsible.unwrap_or("<None>".to_string()),
            expected.unwrap_or("<None>")
        );
    }

    async fn test_completions(args: &str, filter: &str, contains: Vec<&str>) {
        let cmd = CompleteCommand {};
        let cmds = commands();

        let core = Core::builder()
            .with_config(&Config::for_dev_directory(&get_dev_dir()))
            .build();

        let writer: Arc<Mutex<Vec<u8>>> = Arc::new(Mutex::new(Vec::new()));
        let completer = Completer::new_for(filter, writer.clone());
        cmd.offer_completions(&core, &cmds, args, &completer).await;

        let output = String::from_utf8(writer.lock().unwrap().to_vec()).unwrap();

        let mut offers = std::collections::HashSet::new();

        for offer in output.split_terminator("\n") {
            offers.insert(offer);
        }

        for item in contains {
            assert!(
                offers.contains(item),
                "completion output '{}' should contain '{}'",
                output,
                item
            );
        }
    }
}
