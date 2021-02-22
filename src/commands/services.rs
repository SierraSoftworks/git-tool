use super::*;

pub struct ServicesCommand {}

impl Command for ServicesCommand {
    fn name(&self) -> String {
        String::from("services")
    }
    fn app<'a>(&self) -> clap::App<'a> {
        App::new(&self.name())
            .version("1.0")
            .about("list services which can be used with Git-Tool")
            .long_about("Gets the list of services that you have added to your configuration file. These services are responsible for hosting your Git repositories.")
    }
}

#[async_trait]
impl CommandRunnable for ServicesCommand {
    async fn run(
        &self,
        core: &Core,
        _matches: &clap::ArgMatches,
    ) -> Result<i32, crate::core::Error> {
        let mut output = core.output();

        for svc in core.config().get_services() {
            writeln!(output, "{}", svc.get_domain())?;
        }

        Ok(0)
    }

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

        let cmd = ServicesCommand {};
        match cmd.run(&core, &args).await {
            Ok(_) => {}
            Err(err) => panic!("{}", err.message()),
        }

        assert!(
            output.to_string().contains("github.com\n"),
            "the output should contain each service"
        );
    }
}
