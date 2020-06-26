use super::*;
use clap::SubCommand;

pub struct ServicesCommand {

}

#[async_trait]
impl Command for ServicesCommand {
    fn name(&self) -> String {
        String::from("services")
    }
    fn app<'a, 'b>(&self) -> clap::App<'a, 'b> {
        SubCommand::with_name(&self.name())
            .version("1.0")
            .about("list services which can be used with Git-Tool")
            .after_help("Gets the list of services that you have added to your configuration file. These services are responsible for hosting your Git repositories.")
    }
    async fn run<'a>(&self, core: &crate::core::Core, _matches: &clap::ArgMatches<'a>) -> Result<i32, crate::core::Error> {
        for svc in core.config.get_services() {
            println!("{}", svc.get_domain());
        }

        Ok(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::core::{Core, Config};

    #[tokio::test]
    async fn run() {
        let cmd = ServicesCommand{};

        let args = cmd.app().get_matches_from(vec!["services"]);

        let cfg = Config::default();
        let core = Core::builder()
            .with_config(&cfg)
            .build();


        match cmd.run(&core, &args).await {
            Ok(_) => {},
            Err(err) => {
                panic!(err.message())
            }
        }
    }
}