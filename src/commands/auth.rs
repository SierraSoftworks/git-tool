use super::*;
use clap::Arg;
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
    async fn run(&self, core: &Core, matches: &ArgMatches) -> Result<i32, core::Error> {
        let service = matches.get_one::<String>("service").ok_or_else(|| {
            errors::user(
            "You have not provided the name of the service you wish to authenticate.",
            "Please provide the name of the service when running this command: `git-tool auth gh`.",
        )
        })?;

        if let Some(svc) = core.config().get_service(service) {
            if svc.api.is_none() {
                return Err(errors::user(
                    &format!("The service '{}' does not include an API which supports authentication.", &svc.name),
                    "You do not need to configure authentication for this service, but you can check the services in your configuration using `git-tool services`."
                ));
            }

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
                            writeln!(core.output(), "{}", online_service.auth_instructions())?;
                        }

                        writeln!(core.output(), "Access Token: ")?;
                        rpassword::read_password().map_err(|e| errors::user_with_internal(
                        "Could not read the access token that you entered.",
                        "Please try running this command again, or let us know if you continue to run into problems by opening a GitHub issue.",
                        e))?
                    }
                };

                core.keychain().set_token(service, &token)?;

                writeln!(core.output(), "Access Token set for service '{service}'")?;
                if let Some(online_service) = online::services().iter().find(|s| s.handles(svc)) {
                    writeln!(core.output(), "Testing authentication token...")?;
                    online_service.test(core, svc).await?;
                    writeln!(core.output(), "Authentication token is valid.")?;
                }
            }
        } else {
            let suggestion = if let Some(default_service) = core.config().get_services().next() {
                format!("Try running `git-tool auth {default_service}` or use one of the services listed in `git-tool services`.")
            } else {
                "Make sure that you have registered a service in your configuration file.".into()
            };

            return Err(errors::user(
                &format!(
                    "The service you specified ('{service}') does not exist in your configuration."
                ),
                &suggestion,
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
            .with_mock_http_client(vec![core::MockHttpRoute::new(
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
        match cmd.run(&core, &args).await {
            Ok(_) => {}
            Err(err) => panic!("{}", err.message()),
        }
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
        match cmd.run(&core, &args).await {
            Ok(_) => {}
            Err(err) => panic!("{}", err.message()),
        }
    }
}
