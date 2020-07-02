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
    }
}


#[async_trait]
impl<K: KeyChain, L: Launcher, R: Resolver> CommandRunnable<K, L, R> for AuthCommand {
    async fn run<'a>(&self, core: &crate::core::Core<K, L, R>, matches: &clap::ArgMatches<'a>) -> Result<i32, crate::core::Error>
    where K: KeyChain, L: Launcher, R: Resolver {
        let service = matches.value_of("service").ok_or(errors::user(
            "You have not provided the name of the service you wish to authenticate.",
            "Please provide the name of the service when running this command: `git-tool auth github.com`."))?;

        if matches.is_present("remove-token") {
            core.keychain.remove_token(service)?;
        } else {
            let token = rpassword::read_password_from_tty(Some("Access Token: ")).map_err(|e| errors::user_with_internal(
                "Could not read the access token that you entered.",
                "Please try running this command again, or let us know if you continue to run into problems by opening a GitHub issue.",
                e))?;

            core.keychain.set_token(service, &token)?;
        }

        Ok(0)
    }

    async fn complete<'a>(&self, core: &Core<K, L, R>, completer: &Completer, _matches: &ArgMatches<'a>) {
        completer.offer_many(core.config.get_services().map(|s| s.get_domain()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::core::Config;

    #[tokio::test]
    async fn run() {
        
        let cfg = Config::default();
        let core = Core::builder()
        .with_config(&cfg)
        .build();
        
        let cmd = AuthCommand{};
        let args = cmd.app().get_matches_from(vec!["auth", "github.com", "--delete"]);
        match cmd.run(&core, &args).await {
            Ok(_) => {},
            Err(err) => {
                panic!(err.message())
            }
        }
    }
}