use super::app;
use super::Error;
use super::{
    templates::{render, render_list},
    Config, Target,
};
use async_trait::async_trait;
use std::sync::Arc;
use tokio::process::Command;

#[async_trait]
pub trait Launcher: Send + Sync + From<Arc<Config>> {
    async fn run(&self, a: &app::App, t: &(dyn Target + Send + Sync)) -> Result<i32, Error>;
}

pub struct TokioLauncher {
    config: Arc<Config>,
}

impl From<Arc<Config>> for TokioLauncher {
    fn from(config: Arc<Config>) -> Self {
        Self {
            config: config.clone(),
        }
    }
}

#[async_trait]
impl Launcher for TokioLauncher {
    async fn run(&self, a: &app::App, t: &(dyn Target + Send + Sync)) -> Result<i32, Error> {
        let context = t.template_context(&self.config);

        let program = render(a.get_command(), context.clone())?;
        let args = render_list(a.get_args(), context.clone())?;
        let env_args = render_list(a.get_environment(), context.clone())?;
        let env_arg_tuples = env_args
            .iter()
            .map(|i| i.split("=").collect())
            .map(|i: Vec<&str>| (i[0], i[1]));

        let status = Command::new(program)
            .args(args)
            .current_dir(t.get_path())
            .envs(env_arg_tuples)
            .spawn()?
            .await?;

        Ok(status.code().unwrap_or_default())
    }
}

#[cfg(test)]
pub mod mocks {
    use super::*;
    use tokio::sync::Mutex;

    #[derive(Default)]
    pub struct MockLauncher {
        pub launches: Arc<Mutex<Vec<MockLaunch>>>,
        pub status: i32,
        pub return_error: bool,
    }

    impl From<Arc<Config>> for MockLauncher {
        fn from(_: Arc<Config>) -> Self {
            Default::default()
        }
    }

    pub struct MockLaunch {
        pub app: app::App,
        pub target_path: std::path::PathBuf,
    }

    #[async_trait]
    impl Launcher for MockLauncher {
        async fn run(&self, a: &app::App, t: &(dyn Target + Send + Sync)) -> Result<i32, Error> {
            let mut launches = self.launches.lock().await;

            launches.push(MockLaunch {
                app: a.clone(),
                target_path: std::path::PathBuf::from(t.get_path()),
            });

            if self.return_error {
                Err(Error::SystemError(
                    "Mock Error".to_string(),
                    "Configure the mock to not throw an error".to_string(),
                    None,
                ))
            } else {
                Ok(self.status)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::Scratchpad;
    use super::*;
    use crate::test::get_dev_dir;

    #[tokio::test]
    #[cfg(windows)]
    async fn run_app_windows() {
        let a: app::App = app::App::builder()
            .with_name("test")
            .with_command("powershell.exe")
            .with_args(vec![
                "-NoProfile",
                "-NonInteractive",
                "-Command",
                "exit $env:TEST_CODE",
            ])
            .with_environment(vec!["TEST_CODE={{ .Target.Name }}"])
            .into();

        let test_dir = get_dev_dir();
        let t = Scratchpad::new("123", test_dir);

        let config = Arc::new(Config::default());
        let launcher = TokioLauncher::from(config);

        let result = launcher.run(&a, &t).await.unwrap();
        assert_eq!(result, 123);
    }

    #[tokio::test]
    #[cfg(unix)]
    async fn run_app_linux() {
        let a: app::App = app::App::builder()
            .with_name("test")
            .with_command("sh")
            .with_args(vec!["-c", "exit $TEST_CODE"])
            .with_environment(vec!["TEST_CODE={{ .Target.Name }}"])
            .into();

        let test_dir = get_dev_dir();
        let t = Scratchpad::new("123", test_dir);

        let config = Arc::new(Config::default());
        let launcher = TokioLauncher::from(config);

        let result = launcher.run(&a, &t).await.unwrap();
        assert_eq!(result, 123);
    }
}
