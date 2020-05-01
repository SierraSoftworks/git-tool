use async_trait::async_trait;
use tokio::process::Command;
use super::app;
use super::{Error};
use super::Target;

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