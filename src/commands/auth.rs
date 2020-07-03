use super::*;
use clap::{Arg, SubCommand};

pub struct AuthCommand {

}

impl Command for AuthCommand {
    fn name(&self) -> String {
        String::from("auth")
    }
    fn app<'a, 'b>(&self) -> clap::App<'a, 'b> {
        SubCommand::with_name(&self.name())
            .version("1.0")
            .about("configure authentication tokens")
            .after_help("Configures the authentication tokens used by Git-Tool to create and manage your remote repositories.")
            .arg(Arg::with_name("service")
                .index(1)
                .help("the service to configure an access token for")
                .required(true))
            .arg(Arg::with_name("remove-token")
                .long("delete")
                .short("d")
                .help("delete any access token associated with the service"))
                .arg(Arg::with_name("token")
                    .long("token")
                    .help("specifies the token to be set (don't use this unless you have to)")
                    .takes_value(true))
    }
}


#[async_trait]
impl<K: KeyChain, L: Launcher, R: Resolver, O: Output> CommandRunnable<K, L, R, O> for AuthCommand {
    async fn run<'a>(&self, core: &crate::core::Core<K, L, R, O>, matches: &clap::ArgMatches<'a>) -> Result<i32, crate::core::Error>
    where K: KeyChain, L: Launcher, R: Resolver {
        let service = matches.value_of("service").ok_or(errors::user(
            "You have not provided the name of the service you wish to authenticate.",
            "Please provide the name of the service when running this command: `git-tool auth github.com`."))?;

        if matches.is_present("remove-token") {
            core.keychain.remove_token(service)?;
        } else {
            let token = match matches.value_of("token") {
                Some(token) => token.to_string(),
                None => rpassword::read_password_from_tty(Some("Access Token: ")).map_err(|e| errors::user_with_internal(
                    "Could not read the access token that you entered.",
                    "Please try running this command again, or let us know if you continue to run into problems by opening a GitHub issue.",
                    e))?
            };

            core.keychain.set_token(service, &token)?;
        }

        Ok(0)
    }

    async fn complete<'a>(&self, core: &Core<K, L, R, O>, completer: &Completer, _matches: &ArgMatches<'a>) {
        completer.offer_many(core.config.get_services().map(|s| s.get_domain()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::core::Config;

    #[tokio::test]
    async fn run_store() {
        
        let cfg = Config::default();
        let core = Core::builder()
            .with_config(&cfg)
            .with_mock_keychain(|_| {})
            .build();

        assert!(core.keychain.get_token("github.com").is_err());
        
        let cmd = AuthCommand{};
        let args = cmd.app().get_matches_from(vec!["auth", "github.com", "--token", "12345"]);
        match cmd.run(&core, &args).await {
            Ok(_) => {
                assert_eq!(core.keychain.get_token("github.com").unwrap(), "12345");
            },
            Err(err) => {
                panic!(err.message())
            }
        }
    }

    #[tokio::test]
    async fn run_delete() {
        
        let cfg = Config::default();
        let core = Core::builder()
            .with_config(&cfg)
            .with_mock_keychain(|k| {
                k.set_token("github.com", "example").unwrap()
            })
            .build();

        assert!(core.keychain.get_token("github.com").is_ok());
        
        let cmd = AuthCommand{};
        let args = cmd.app().get_matches_from(vec!["auth", "github.com", "--delete"]);
        match cmd.run(&core, &args).await {
            Ok(_) => {
                assert!(core.keychain.get_token("github.com").is_err());
            },
            Err(err) => {
                panic!(err.message())
            }
        }
    }
}