use super::*;
use clap::SubCommand;

pub struct AppsCommand {

}

#[async_trait]
impl Command for AppsCommand {
    fn name(&self) -> String {
        String::from("apps")
    }
    fn app<'a, 'b>(&self) -> clap::App<'a, 'b> {
        SubCommand::with_name(&self.name())
            .version("1.0")
            .about("list applications which can be run through Git-Tool")
            .after_help("Gets the list of applications that you have added to your configuration file. These applications can be run through the `open` and `scratch` commands.")
    }
    async fn run<'a>(&self, core: &crate::core::Core, _matches: &clap::ArgMatches<'a>) -> Result<i32, crate::core::Error> {
        for app in core.config.get_apps() {
            println!("{}", app.get_name());
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
        let cmd = AppsCommand{};

        let args = cmd.app().get_matches_from(vec!["apps"]);

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