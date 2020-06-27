use async_trait::async_trait;
use tokio::process::Command;
use super::app;
use super::Error;
use super::{Config, Target};

#[cfg(test)]
use tokio::sync::Mutex;
#[cfg(test)]
use std::sync::Arc;

#[async_trait]
pub trait Launcher: Send + Sync + From<Config> {
    async fn run(&self, a: &app::App, t: &(dyn Target + Send + Sync)) -> Result<i32, Error>;
}

pub struct TokioLauncher {}

impl From<Config> for TokioLauncher {
    fn from(_: Config) -> Self {
        Self{}
    }
}

#[async_trait]
impl Launcher for TokioLauncher {
    async fn run(&self, a: &app::App, t: &(dyn Target + Send + Sync)) -> Result<i32, Error> {
        let status = Command::new(a.get_command())
            .args(a.get_args())
            .current_dir(t.get_path())
            .envs(a.get_environment().iter().map(|i| i.split("=").collect()).map(|i: Vec<&str>| (i[0], i[1])))
            .spawn()?
            .await?;

        Ok(status.code().unwrap_or_default())
    }
}

#[cfg(test)]
#[derive(Default)]
pub struct MockLauncher {
    pub launches: Arc<Mutex<Vec<MockLaunch>>>,
    pub status: i32,
    pub error: Option<Error>
}

#[cfg(test)]
impl From<Config> for MockLauncher {
    fn from(_: Config) -> Self {
        Default::default()
    }
}

#[cfg(test)]
pub struct MockLaunch {
    pub app: app::App,
    pub target_path: std::path::PathBuf
}

#[cfg(test)]
#[async_trait]
impl Launcher for MockLauncher {
    async fn run(&self, a: &app::App, t: &(dyn Target + Send + Sync)) -> Result<i32, Error> {
        let mut launches = self.launches.lock().await;

        launches.push(MockLaunch{
            app: a.clone(),
            target_path: std::path::PathBuf::from(t.get_path())
        });

        match self.error.clone() {
            Some(e) => Err(e),
            None => {
                Ok(self.status)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::Scratchpad;
    use std::path::PathBuf;

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
                "exit $env:TEST_CODE"
            ])
            .with_environment(vec!["TEST_CODE=123"])
            .into();

        let test_dir = PathBuf::from(file!())
            .parent()
            .and_then(|f| f.parent())
            .and_then(|f| f.parent())
            .and_then(|f| Some(f.join("test")))
            .unwrap();

        let t = Scratchpad::new("test", test_dir);

        let launcher = TokioLauncher{};

        let result = launcher.run(&a, &t).await.unwrap();
        assert_eq!(result, 123);
    }

    #[tokio::test]
    #[cfg(unix)]
    async fn run_app_linux() {
        let a: app::App = app::App::builder()
            .with_name("test")
            .with_command("sh")
            .with_args(vec![
                "-c",
                "exit $TEST_CODE"
            ])
            .with_environment(vec!["TEST_CODE=123"])
            .into();

        let test_dir = PathBuf::from(file!())
            .parent()
            .and_then(|f| f.parent())
            .and_then(|f| f.parent())
            .and_then(|f| Some(f.join("test")))
            .unwrap();

        let t = Scratchpad::new("test", test_dir);

        let launcher = TokioLauncher{};

        let result = launcher.run(&a, &t).await.unwrap();
        assert_eq!(result, 123);
    }
}