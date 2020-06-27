use clap::{App, SubCommand, ArgMatches};
use super::Command;
use super::*;
use super::async_trait;

pub struct ConfigCommand {

}

impl Command for ConfigCommand {
    fn name(&self) -> String {
        String::from("config")
    }

    fn app<'a, 'b>(&self) -> App<'a, 'b> {
        SubCommand::with_name(self.name().as_str())
            .version("1.0")
            .about("manage your Git-Tool configuration file")
            .help_message("This tool allows you to easily make changes to your Git-Tool config file.")
    }
}
    
#[async_trait]
impl<F: FileSource, L: Launcher, R: Resolver> CommandRun<F, L, R> for ConfigCommand {
    async fn run<'a>(&self, core: &core::Core<F, L, R>, _matches: &ArgMatches<'a>) -> Result<i32, errors::Error> {
        core.config.to_writer(std::io::stdout())?;

        Ok(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::core::{Config};
    use clap::ArgMatches;

    #[tokio::test]
    async fn run() {
        let args = ArgMatches::default();
        let cfg = Config::from_str("directory: /dev").unwrap();
        let core = Core::builder().with_config(&cfg).build();

        let cmd = ConfigCommand{};

        match cmd.run(&core, &args).await {
            Ok(_) => {},
            Err(err) => {
                panic!(err.message())
            }
        }
    }
}