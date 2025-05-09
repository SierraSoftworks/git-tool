use super::*;
use tracing_batteries::prelude::*;

pub struct ServicesCommand;
crate::command!(ServicesCommand);

#[async_trait]
impl CommandRunnable for ServicesCommand {
    fn name(&self) -> String {
        String::from("services")
    }
    fn app(&self) -> clap::Command {
        clap::Command::new(self.name())
            .version("1.0")
            .about("list services which can be used with Git-Tool")
            .long_about("Gets the list of services that you have added to your configuration file. These services are responsible for hosting your Git repositories.")
    }

    #[tracing::instrument(name = "gt services", err, skip(self, core, _matches))]
    async fn run(&self, core: &Core, _matches: &ArgMatches) -> Result<i32, engine::Error> {
        let mut output = core.output();

        for svc in core.config().get_services() {
            writeln!(output, "{}", &svc.name)?;
        }

        Ok(0)
    }

    #[tracing::instrument(
        name = "gt complete -- gt services",
        skip(self, _core, _completer, _matches)
    )]
    async fn complete(&self, _core: &Core, _completer: &Completer, _matches: &ArgMatches) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::console::MockConsoleProvider;
    use std::sync::Arc;

    #[tokio::test]
    async fn run() {
        let args = ArgMatches::default();

        let console = Arc::new(MockConsoleProvider::new());
        let core = Core::builder()
            .with_default_config()
            .with_console(console.clone())
            .build();

        let cmd = ServicesCommand {};
        cmd.assert_run_successful(&core, &args).await;

        assert!(
            console.to_string().contains("gh\n"),
            "the output should contain each service"
        );
    }
}
