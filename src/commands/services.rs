use super::*;
use clap::SubCommand;

pub struct ServicesCommand {

}

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
}

#[async_trait]
impl<F: FileSource, L: Launcher, R: Resolver> CommandRunnable<F, L, R> for ServicesCommand {
    async fn run<'a>(&self, core: &crate::core::Core<F, L, R>, _matches: &clap::ArgMatches<'a>) -> Result<i32, crate::core::Error> {
        for svc in core.config.get_services() {
            println!("{}", svc.get_domain());
        }

        Ok(0)
    }

    async fn complete<'a>(&self, _core: &Core<F, L, R>, _completer: &Completer, _matches: &ArgMatches<'a>) {
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::core::{Core, Config};

    #[tokio::test]
    async fn run() {
        let args = ArgMatches::default();
        
        let cfg = Config::default();
        let core = Core::builder()
        .with_config(&cfg)
        .build();
        
        let cmd = ServicesCommand{};
        match cmd.run(&core, &args).await {
            Ok(_) => {},
            Err(err) => {
                panic!(err.message())
            }
        }
    }
}