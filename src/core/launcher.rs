use async_trait::async_trait;
use tokio::process::Command;
use super::app;
use super::Error;
use super::Target;

#[cfg(test)]
use tokio::sync::Mutex;
#[cfg(test)]
use std::sync::Arc;

#[async_trait]
pub trait Launcher {
    async fn run(&self, a: &app::App, t: &(dyn Target + Send + Sync)) -> Result<i32, Error>;
}

pub struct TokioLauncher {}

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
pub struct MockLauncher {
    pub launches: Arc<Mutex<Vec<MockLaunch>>>,
    pub status: Arc<Mutex<i32>>,
    pub error: Arc<Mutex<Option<Error>>>
}

#[cfg(test)]
impl Default for MockLauncher {
    fn default() -> Self {
        Self {
            launches: Arc::new(Mutex::new(Vec::new())),
            status: Arc::new(Mutex::new(0)),
            error: Arc::new(Mutex::new(None))
        }
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

        let err = self.error.lock().await;
        
        match err.clone() {
            Some(e) => Err(e.clone()),
            None => {
                let status = self.status.lock().await;

                Ok(*status)
            }
        }
    }
}