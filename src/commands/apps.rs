use super::*;

pub struct AppsCommand {}

impl Command for AppsCommand {
    fn name(&self) -> String {
        String::from("apps")
    }
    fn app<'a>(&self) -> clap::App<'a> {
        App::new(&self.name())
            .version("1.0")
            .about("list applications which can be run through Git-Tool")
            .long_about("Gets the list of applications that you have added to your configuration file. These applications can be run through the `open` and `scratch` commands.")
    }
}

#[async_trait]
impl<C: Core> CommandRunnable<C> for AppsCommand {
    async fn run(&self, core: &C, _matches: &clap::ArgMatches) -> Result<i32, crate::core::Error> {
        for app in core.config().get_apps() {
            writeln!(core.output().writer(), "{}", app.get_name())?;
        }

        Ok(0)
    }

    async fn complete(&self, _core: &C, _completer: &Completer, _matches: &ArgMatches) {}
}

#[cfg(test)]
mod tests {
    use super::core::{Config, CoreBuilder};
    use super::*;

    #[tokio::test]
    async fn run() {
        let args = ArgMatches::default();

        let cfg = Config::default();
        let core = CoreBuilder::default()
            .with_config(&cfg)
            .with_mock_output()
            .build();

        let cmd = AppsCommand {};
        match cmd.run(&core, &args).await {
            Ok(_) => {}
            Err(err) => panic!(err.message()),
        }

        let output = core.output().to_string();
        assert!(
            output.contains("shell"),
            "the output should contain the default app"
        );
    }
}
