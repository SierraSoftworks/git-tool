use super::*;
use tracing_batteries::prelude::*;

pub struct DoctorCommand;
crate::command!(DoctorCommand);

#[async_trait]
impl CommandRunnable for DoctorCommand {
    fn name(&self) -> String {
        String::from("doctor")
    }

    fn app(&self) -> clap::Command {
        clap::Command::new(self.name())
            .version("1.0")
            .about("checks that your environment is configured correctly for Git-Tool")
            .long_about("Runs a series of checks to ensure that the environment is ready to run the application")
    }

    #[tracing::instrument(name = "gt doctor", err, skip(self, core, _matches))]
    async fn run(&self, core: &Core, _matches: &ArgMatches) -> Result<i32, core::Error> {
        writeln!(core.output(), "Checking environment...")?;

        if core.config().file_exists() {
            writeln!(core.output(), "[OK] Config file exists")?;
        } else {
            writeln!(
                core.output(),
                "[WARNING] Config file does not exist, you are using the built-in defaults"
            )?;
        }

        if !core.config().get_dev_directory().exists() {
            Err(errors::user(
                "Your development directory does not exist.",
                "Make sure that the dev directory you have specified in your configuration file exists and is writable by Git-Tool.",
            ))?;
        }

        writeln!(core.output(), "[OK] Development directory exists")?;

        if !core.config().get_scratch_directory().exists() {
            Err(errors::user(
                "Your scratch directory does not exist.",
                "Make sure that the scratch directory you have specified in your configuration file exists and is writable by Git-Tool.",
            ))?;
        }

        for svc in core.config().get_services() {
            if let Some(online_service) = online::services().iter().find(|s| s.handles(svc)) {
                match online_service.test(core, svc).await {
                    Ok(_) => {
                        writeln!(core.output(), "[OK] Access to '{}' is working", &svc.name)?;
                    }
                    Err(err) => {
                        writeln!(
                            core.output(),
                            "[ERROR] Access to '{}' is not working, run `git-tool auth {}` to fix it: {}",
                            &svc.name,
                            &svc.name,
                            err
                        )?;
                    }
                }
            }
        }

        Ok(0)
    }

    #[tracing::instrument(
        name = "gt complete -- gt doctor",
        skip(self, _core, _completer, _matches)
    )]
    async fn complete(&self, _core: &Core, _completer: &Completer, _matches: &ArgMatches) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[cfg_attr(feature = "pure-tests", ignore)]
    async fn run() {
        let args = ArgMatches::default();

        let temp = tempfile::tempdir().unwrap();

        let console = crate::console::mock();
        let core = Core::builder()
            .with_config_for_dev_directory(temp.path())
            .with_console(console.clone())
            .build();

        std::fs::create_dir_all(core.config().get_scratch_directory()).unwrap();

        std::env::set_var(
            "GITTOOL_CONFIG",
            temp.path().join("config.yml").to_str().unwrap(),
        );

        // Ensure that the config file is created
        core.config()
            .save(temp.path().join("config.yml"))
            .await
            .unwrap();

        let cmd = DoctorCommand {};
        match cmd.run(&core, &args).await {
            Ok(_) => {}
            Err(err) => panic!("{}", err.message()),
        }

        assert!(
            console.to_string().contains("Checking environment..."),
            "the output should contain the default app"
        );
    }
}
