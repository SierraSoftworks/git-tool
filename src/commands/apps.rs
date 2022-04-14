use super::*;

pub struct AppsCommand {}

impl Command for AppsCommand {
    fn name(&self) -> String {
        String::from("apps")
    }
    fn app<'a>(&self) -> clap::Command<'a> {
        clap::Command::new(&self.name())
            .version("1.0")
            .about("list applications which can be run through Git-Tool")
            .long_about("Gets the list of applications that you have added to your configuration file. These applications can be run through the `open` and `scratch` commands.")
    }
}

#[async_trait]
impl CommandRunnable for AppsCommand {
    #[tracing::instrument(name = "gt apps", err, skip(self, core, _matches))]
    async fn run(
        &self,
        core: &Core,
        _matches: &clap::ArgMatches,
    ) -> Result<i32, crate::core::Error> {
        for app in core.config().get_apps() {
            writeln!(core.output(), "{}", app.get_name())?;
        }

        Ok(0)
    }

    #[tracing::instrument(
        name = "gt complete -- gt apps",
        skip(self, _core, _completer, _matches)
    )]
    async fn complete(&self, _core: &Core, _completer: &Completer, _matches: &ArgMatches) {}
}

#[cfg(test)]
mod tests {
    use super::core::Config;
    use super::*;

    #[tokio::test]
    async fn run() {
        let args = ArgMatches::default();

        let cfg = Config::default();
        let core = Core::builder().with_config(&cfg).build();

        let output = crate::console::output::mock();

        let cmd = AppsCommand {};
        match cmd.run(&core, &args).await {
            Ok(_) => {}
            Err(err) => panic!("{}", err.message()),
        }

        assert!(
            output.to_string().contains("shell"),
            "the output should contain the default app"
        );
    }
}
