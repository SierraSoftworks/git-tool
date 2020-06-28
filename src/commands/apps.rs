use super::*;
use clap::SubCommand;

pub struct AppsCommand {

}

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
}


#[async_trait]
impl<F: FileSource, L: Launcher, R: Resolver> CommandRunnable<F, L, R> for AppsCommand {
    async fn run<'a>(&self, core: &crate::core::Core<F, L, R>, _matches: &clap::ArgMatches<'a>) -> Result<i32, crate::core::Error>
    where F: FileSource, L: Launcher, R: Resolver {
        for app in core.config.get_apps() {
            println!("{}", app.get_name());
        }

        Ok(0)
    }

    async fn complete<'a>(&self, _core: &Core<F, L, R>, _completer: &Completer, _matches: &ArgMatches<'a>) {
        
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::core::Config;

    #[tokio::test]
    async fn run() {
        let args = ArgMatches::default();
        
        let cfg = Config::default();
        let core = Core::builder()
        .with_config(&cfg)
        .build();
        
        let cmd = AppsCommand{};
        match cmd.run(&core, &args).await {
            Ok(_) => {},
            Err(err) => {
                panic!(err.message())
            }
        }
    }
}