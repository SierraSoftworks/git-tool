use crate::errors::HumanErrorResultExt;

use super::*;
use clap::Arg;
use human_errors::prelude::*;
use tracing_batteries::prelude::*;

pub struct AuthCommand;

crate::command!(AuthCommand);

#[async_trait]
impl CommandRunnable for AuthCommand {
    fn name(&self) -> String {
        String::from("auth")
    }
    fn app(&self) -> clap::Command {
        clap::Command::new(self.name())
            .version("1.0")
            .about("configure authentication tokens")
            .long_about("Configures the authentication tokens used by Git-Tool to create and manage your remote repositories.")
            .arg(Arg::new("service")
                .index(1)
                .help("the service to configure an access token for")
                .required(true))
            .arg(Arg::new("remove-token")
                .long("delete")
                .short('d')
                .help("delete any access token associated with the service")
                .action(clap::ArgAction::SetTrue))
            .arg(Arg::new("token")
                .long("token")
                .help("specifies the token to be set (don't use this unless you have to)")
                .action(clap::ArgAction::Set))
    }

    #[tracing::instrument(name = "gt auth", err, skip(self, core, matches))]
    async fn run(&self, core: &Core, matches: &ArgMatches) -> Result<i32, engine::Error> {
        let service = matches.get_one::<String>("service").ok_or_else(|| {
            human_errors::user("You have not provided the name of the service you wish to authenticate.", &["Please provide the name of the service when running this command: `git-tool auth gh`."])
        })?;

        if let Ok(svc) = core.config().get_service(service) {
            svc.api.as_ref().ok_or_user_err(
                format!(
                    "The service '{}' does not include an API which supports authentication.",
                    &svc.name
                ),
                &["You do not need to configure authentication for this service, but you can check the services in your configuration using `git-tool services`."],
            )?;

            if matches.get_flag("remove-token") {
                core.keychain().delete_token(service)?;
            } else {
                let token = match matches.get_one::<String>("token") {
                    Some(token) => token.to_string(),
                    None => {
                        if let Some(online_service) = online::service::services()
                            .iter()
                            .find(|s| s.handles(svc))
                            .cloned()
                        {
                            writeln!(core.output(), "{}", online_service.auth_instructions())
                                .to_human_error()?;
                        }

                        writeln!(core.output(), "Access Token: ").to_human_error()?;
                        rpassword::read_password().wrap_user_err(
                            "Could not read the access token that you entered.",
                            &["Please try running this command again, or let us know if you continue to run into problems by opening a GitHub issue."],
                        )?
                    }
                };

                core.keychain().set_token(service, &token)?;

                writeln!(core.output(), "Access Token set for service '{service}'")
                    .to_human_error()?;
                if let Some(online_service) = online::services().iter().find(|s| s.handles(svc)) {
                    writeln!(core.output(), "Testing authentication token...").to_human_error()?;
                    online_service.test(core, svc).await?;
                    writeln!(core.output(), "Authentication token is valid.").to_human_error()?;
                }
            }
        } else if core.config().get_services().next().is_some() {
            return Err(human_errors::user(
                format!(
                    "The service you specified ('{service}') does not exist in your configuration."
                ),
                &["Try using one of the services listing in `git-tool services` instead."],
            ));
        } else {
            return Err(human_errors::user(
                format!(
                    "The service you specified ('{service}') does not exist in your configuration."
                ),
                &["Make sure that you have registered a service in your configuration file."],
            ));
        }

        Ok(0)
    }

    #[tracing::instrument(name = "gt complete -- gt auth", skip(self, core, completer, _matches))]
    async fn complete(&self, core: &Core, completer: &Completer, _matches: &ArgMatches) {
        completer.offer_many(core.config().get_services().map(|s| &s.name));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;

    #[tokio::test]
    async fn run_store() {
        let core = Core::builder()
            .with_default_config()
            .with_mock_keychain(|mock| {
                mock.expect_set_token()
                    .with(eq("gh"), eq("mock-token"))
                    .times(1)
                    .returning(|_, _| Ok(()));
                mock.expect_get_token()
                    .with(eq("gh"))
                    .times(1)
                    .returning(|_| Ok("mock-token".to_string()));
            })
            .with_null_console()
            .with_mock_http_client(vec![engine::MockHttpRoute::new(
                "GET",
                "https://api.github.com/user",
                200,
                r#"{"login": "test"}"#,
            )])
            .build();

        let cmd = AuthCommand {};
        let args = cmd
            .app()
            .get_matches_from(vec!["auth", "gh", "--token", "mock-token"]);
        cmd.assert_run_successful(&core, &args).await;
    }

    #[tokio::test]
    async fn run_delete() {
        let core = Core::builder()
            .with_default_config()
            .with_null_console()
            .with_mock_keychain(|mock| {
                mock.expect_delete_token()
                    .with(eq("gh"))
                    .times(1)
                    .returning(|_| Ok(()));
            })
            .build();

        let cmd = AuthCommand {};
        let args = cmd.app().get_matches_from(vec!["auth", "gh", "--delete"]);
        cmd.assert_run_successful(&core, &args).await;
    }
}
