use super::*;
use std::env;

pub struct DoctorCommand {}

impl Command for DoctorCommand {
    fn name(&self) -> String {
        String::from("doctor")
    }

    fn app<'a>(&self) -> clap::Command<'a> {
        clap::Command::new(&self.name())
            .version("1.0")
            .about("checks that your environment is configured correctly for Git-Tool")
            .long_about("Runs a series of checks to ensure that the environment is ready to run the application")
    }
}

#[async_trait]
impl CommandRunnable for DoctorCommand {
    async fn run(
        &self,
        core: &Core,
        _matches: &clap::ArgMatches,
    ) -> Result<i32, crate::core::Error> {
        let mut output = core.output();

        writeln!(output, "Checking environment...")?;

        let config_path = env::var("GITTOOL_CONFIG").map_err(|_| {
            errors::user(
                "GITTOOL_CONFIG environment variable is not set",
                "Set the GITTOOL_CONFIG environment variable to the path of your config file",
            )
        })?;

        writeln!(output, "[OK] GITTOOL_CONFIG environment variable set")?;

        if !std::path::Path::new(&config_path).exists() {
            Err(errors::user(
                "GITTOOL_CONFIG environment variable is set to a path that does not exist",
                "Set the GITTOOL_CONFIG environment variable to the path of your config file",
            ))?;
        }

        writeln!(
            output,
            "[OK] GITTOOL_CONFIG environment variable points to a valid file"
        )?;

        match core.config().get_config_file() {
            Some(config_file) if config_file != std::path::Path::new(&config_path) => {
                Err(errors::user(
                    "GITTOOL_CONFIG environment variable is set to a path that does not match the config file",
                    "Set the GITTOOL_CONFIG environment variable to the path of your config file",
                ))?;
            }
            _ => {}
        }

        writeln!(output, "[OK] GITTOOL_CONFIG value was loaded at startup")?;

        if !core.config().get_dev_directory().exists() {
            Err(errors::user(
                "Your development directory does not exist.",
                "Make sure that the dev directory you have specified in your configuration file exists and is writable by Git-Tool.",
            ))?;
        }

        writeln!(output, "[OK] Development directory exists")?;

        if !core.config().get_scratch_directory().exists() {
            Err(errors::user(
                "Your scratch directory does not exist.",
                "Make sure that the scratch directory you have specified in your configuration file exists and is writable by Git-Tool.",
            ))?;
        }

        for svc in core.config().get_services() {
            if let Some(online_service) = crate::online::services().iter().find(|s| s.handles(svc))
            {
                match online_service.test(core).await {
                    Ok(_) => {
                        writeln!(output, "[OK] Access to '{}' is working", svc.get_domain())?;
                    }
                    Err(err) => {
                        writeln!(
                            output,
                            "[ERROR] Access to '{}' is not working, run `git-tool auth {}` to fix it: {}",
                            svc.get_domain(),
                            svc.get_domain(),
                            err
                        )?;
                    }
                }
            }
        }

        Ok(0)
    }

    async fn complete(&self, _core: &Core, _completer: &Completer, _matches: &clap::ArgMatches) {}
}

#[cfg(test)]
mod tests {
    use super::core::Config;
    use super::*;

    #[tokio::test]
    async fn run() {
        let args = ArgMatches::default();

        let temp = tempfile::tempdir().unwrap();
        let cfg = Config::for_dev_directory(temp.path());
        std::fs::create_dir_all(cfg.get_scratch_directory()).unwrap();

        let core = Core::builder().with_config(&cfg).build();

        let output = crate::console::output::mock();

        std::env::set_var(
            "GITTOOL_CONFIG",
            temp.path().join("config.yml").to_str().unwrap(),
        );

        // Ensure that the config file is created
        std::fs::write(
            temp.path().join("config.yml"),
            serde_yaml::to_string(&cfg).unwrap(),
        )
        .unwrap();

        let cmd = DoctorCommand {};
        match cmd.run(&core, &args).await {
            Ok(_) => {}
            Err(err) => panic!("{}", err.message()),
        }

        assert!(
            output.to_string().contains("Checking environment..."),
            "the output should contain the default app"
        );
    }
}
