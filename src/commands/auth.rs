use super::*;
use clap::Arg;

pub struct AuthCommand {}

impl Command for AuthCommand {
    fn name(&self) -> String {
        String::from("auth")
    }
    fn app<'a>(&self) -> clap::Command<'a> {
        clap::Command::new(&self.name())
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
                .help("delete any access token associated with the service"))
                .arg(Arg::new("token")
                    .long("token")
                    .help("specifies the token to be set (don't use this unless you have to)")
                    .takes_value(true))
    }
}

#[async_trait]
impl CommandRunnable for AuthCommand {
    #[tracing::instrument(name = "gt auth", err, skip(self, core, matches))]
    async fn run(
        &self,
        core: &Core,
        matches: &clap::ArgMatches,
    ) -> Result<i32, crate::core::Error> {
        let service = matches.value_of("service").ok_or_else(|| {
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

            if matches.is_present("remove-token") {
                core.keychain().remove_token(service)?;
            } else {
                let token = match matches.value_of("token") {
                    Some(token) => token.to_string(),
                    None => {
                        writeln!(core.output(), "Access Token: ")?;
                        rpassword::read_password().map_err(|e| errors::user_with_internal(
                        "Could not read the access token that you entered.",
                        "Please try running this command again, or let us know if you continue to run into problems by opening a GitHub issue.",
                        e))?
                    }
                };

                core.keychain().set_token(service, &token)?;

                writeln!(core.output(), "Access Token set for service '{}'", service)?;
                if let Some(online_service) =
                    crate::online::services().iter().find(|s| s.handles(svc))
                {
                    writeln!(core.output(), "Testing authentication token...")?;
                    online_service.test(core, svc).await?;
                    writeln!(core.output(), "Authentication token is valid.")?;
                }
            }
        } else {
            return Err(errors::user(
                "The service you specified does not exist in your configuration.",
                "Please run `git-tool services` to see a list of available services.",
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
    use super::core::Config;
    use super::*;
    use mocktopus::mocking::*;

    #[tokio::test]
    async fn run_store() {
        let cfg = Config::default();
        let core = Core::builder().with_config(&cfg).build();

        crate::console::output::mock();
        core::KeyChain::set_token.mock_safe(|_, token, value| {
            assert_eq!(token, "gh", "the correct token should be saved");
            assert_eq!(value, "12345", "the correct value should be saved");
            MockResult::Return(Ok(()))
        });
        core::KeyChain::get_token.mock_safe(|_, token| {
            assert_eq!(token, "gh", "the correct token should be retrieved");
            MockResult::Return(Ok("".into()))
        });

        core::HttpClient::mock(vec![core::HttpClient::route(
            "GET",
            "https://api.github.com/user",
            200,
            r#"{"login":"test"}"#,
        )]);

        let cmd = AuthCommand {};
        let args = cmd
            .app()
            .get_matches_from(vec!["auth", "gh", "--token", "12345"]);
        match cmd.run(&core, &args).await {
            Ok(_) => {}
            Err(err) => panic!("{}", err.message()),
        }
    }

    #[tokio::test]
    async fn run_delete() {
        let cfg = Config::default();
        let core = Core::builder().with_config(&cfg).build();

        crate::console::output::mock();
        core::KeyChain::remove_token.mock_safe(|_, token| {
            assert_eq!(token, "gh", "the correct token should be removed");
            MockResult::Return(Ok(()))
        });

        let cmd = AuthCommand {};
        let args = cmd.app().get_matches_from(vec!["auth", "gh", "--delete"]);
        match cmd.run(&core, &args).await {
            Ok(_) => {}
            Err(err) => panic!("{}", err.message()),
        }
    }
}
