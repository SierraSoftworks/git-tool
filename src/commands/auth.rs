use super::*;
use clap::Arg;

pub struct AuthCommand {}

impl Command for AuthCommand {
    fn name(&self) -> String {
        String::from("auth")
    }
    fn app<'a>(&self) -> clap::App<'a> {
        App::new(&self.name())
            .version("1.0")
            .about("configure authentication tokens")
            .long_about("Configures the authentication tokens used by Git-Tool to create and manage your remote repositories.")
            .arg(Arg::new("service")
                .index(1)
                .about("the service to configure an access token for")
                .required(true))
            .arg(Arg::new("remove-token")
                .long("delete")
                .short('d')
                .about("delete any access token associated with the service"))
                .arg(Arg::new("token")
                    .long("token")
                    .about("specifies the token to be set (don't use this unless you have to)")
                    .takes_value(true))
    }
}

#[async_trait]
impl<C: Core> CommandRunnable<C> for AuthCommand {
    async fn run(&self, core: &C, matches: &clap::ArgMatches) -> Result<i32, crate::core::Error>
    where
        C: Core,
    {
        let service = matches.value_of("service").ok_or(errors::user(
            "You have not provided the name of the service you wish to authenticate.",
            "Please provide the name of the service when running this command: `git-tool auth github.com`."))?;

        if matches.is_present("remove-token") {
            core.keychain().remove_token(service)?;
        } else {
            let token = match matches.value_of("token") {
                Some(token) => token.to_string(),
                None => rpassword::read_password_from_tty(Some("Access Token: ")).map_err(|e| errors::user_with_internal(
                    "Could not read the access token that you entered.",
                    "Please try running this command again, or let us know if you continue to run into problems by opening a GitHub issue.",
                    e))?
            };

            core.keychain().set_token(service, &token)?;
        }

        Ok(0)
    }

    async fn complete(&self, core: &C, completer: &Completer, _matches: &ArgMatches) {
        completer.offer_many(core.config().get_services().map(|s| s.get_domain()));
    }
}

#[cfg(test)]
mod tests {
    use super::core::{Config, CoreBuilder};
    use super::*;

    #[tokio::test]
    async fn run_store() {
        let cfg = Config::default();
        let core = CoreBuilder::default()
            .with_config(&cfg)
            .with_mock_output()
            .with_mock_keychain(|_| {})
            .build();

        assert!(core.keychain().get_token("github.com").is_err());

        let cmd = AuthCommand {};
        let args = cmd
            .app()
            .get_matches_from(vec!["auth", "github.com", "--token", "12345"]);
        match cmd.run(&core, &args).await {
            Ok(_) => {
                assert_eq!(core.keychain().get_token("github.com").unwrap(), "12345");
            }
            Err(err) => panic!(err.message()),
        }
    }

    #[tokio::test]
    async fn run_delete() {
        let cfg = Config::default();
        let core = CoreBuilder::default()
            .with_config(&cfg)
            .with_mock_output()
            .with_mock_keychain(|k| k.set_token("github.com", "example").unwrap())
            .build();

        assert!(core.keychain().get_token("github.com").is_ok());

        let cmd = AuthCommand {};
        let args = cmd
            .app()
            .get_matches_from(vec!["auth", "github.com", "--delete"]);
        match cmd.run(&core, &args).await {
            Ok(_) => {
                assert!(core.keychain().get_token("github.com").is_err());
            }
            Err(err) => panic!(err.message()),
        }
    }
}
