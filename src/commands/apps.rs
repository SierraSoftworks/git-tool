use super::*;
use tracing_batteries::prelude::*;

pub struct AppsCommand;

crate::command!(AppsCommand);

#[async_trait]
impl CommandRunnable for AppsCommand {
    fn name(&self) -> String {
        String::from("apps")
    }
    fn app(&self) -> clap::Command {
        clap::Command::new(self.name())
            .version("1.0")
            .about("list applications which can be run through Git-Tool")
            .long_about("Gets the list of applications that you have added to your configuration file. These applications can be run through the `open` and `scratch` commands.")
    }

    #[tracing::instrument(name = "gt apps", err, skip(self, core, _matches))]
    async fn run(&self, core: &Core, _matches: &ArgMatches) -> Result<i32, engine::Error> {
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
    use super::*;

    #[tokio::test]
    async fn run() {
        let args = ArgMatches::default();

        let console = crate::console::mock();
        let core = Core::builder()
            .with_default_config()
            .with_console(console.clone())
            .build();

        let cmd = AppsCommand {};
        cmd.assert_run_successful(&core, &args).await;

        assert!(
            console.to_string().contains("shell"),
            "the output should contain the default app"
        );
    }
}
