use crate::errors;

use super::Error;
use super::app;
use super::{
    Config, Target,
    templates::{render, render_list},
};
use futures::{FutureExt, pin_mut};
use tracing_batteries::prelude::*;

#[cfg(test)]
use mockall::automock;

use std::sync::Arc;
use tokio::process::Command;

#[async_trait::async_trait]
#[cfg_attr(test, automock)]
pub trait Launcher: Send + Sync {
    async fn run(&self, a: &app::App, t: &(dyn Target + Send + Sync)) -> Result<i32, Error>;
}

pub fn launcher(config: Arc<Config>) -> Arc<dyn Launcher + Send + Sync> {
    Arc::new(TrueLauncher { config })
}

struct TrueLauncher {
    config: Arc<Config>,
}

impl std::fmt::Debug for dyn Launcher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Launcher")
    }
}

#[async_trait::async_trait]
impl Launcher for TrueLauncher {
    #[tracing::instrument(name = "launch", err, skip(self, t, a), fields(app=%a, target=%t))]
    async fn run(&self, a: &app::App, t: &(dyn Target + Send + Sync)) -> Result<i32, Error> {
        let context = t.template_context(&self.config);

        let program = render(a.get_command(), context.clone())?;
        let args = render_list(a.get_args(), context.clone())?;
        let env_args = render_list(a.get_environment(), context.clone())?;
        let env_arg_tuples = env_args
            .iter()
            .map(|i| i.split('=').collect())
            .map(|i: Vec<&str>| (i[0], i[1]));

        let mut child = Command::new(program)
            .args(args)
            .current_dir(t.get_path())
            .envs(env_arg_tuples)
            .spawn()
            .map_err(|err| errors::user_with_internal(
                &format!("Could not launch the application '{}' due to an OS-level error.", a.get_command()),
                "Make sure that the program exists on your $PATH and is executable before trying again.",
                err
            ))?;

        self.forward_signals(&mut child).await
    }
}

impl TrueLauncher {
    #[cfg(windows)]
    async fn forward_signals(&self, child: &mut tokio::process::Child) -> Result<i32, Error> {
        loop {
            let ctrlc = tokio::signal::ctrl_c().fuse();
            pin_mut!(ctrlc);

            tokio::select! {
                _ = ctrlc => {
                    // We capture the Ctrl+C signal and ignore it so that the child process
                    // can handle it as necessary.
                },
                status = child.wait() => {
                    return Ok(status.map_err(|err| errors::system_with_internal(
                        "We could not determine the exit status code for the program you ran.",
                        "Please report this error to us on GitHub so that we can work with you to investigate the cause.",
                        err
                    ))?.code().unwrap_or_default());
                }
            }
        }
    }

    #[cfg(unix)]
    async fn forward_signals(&self, child: &mut tokio::process::Child) -> Result<i32, Error> {
        let child_id = child.id().ok_or_else(|| errors::user(
            "Unable to determine the child process's PID because the child process has already exited.",
            "This might not be a problem, depending on the program you are running, however it may also indicate that the process is not running correctly."
        ))?;

        let pid = nix::unistd::Pid::from_raw(child_id.try_into().map_err(|err| errors::system_with_internal(
            "Unable to convert child process ID to a valid PID. This may impact Git-Tool's ability to forward signals to a child application.",
            "Please report this error to us on GitHub, along with information about your operating system and version of Git-Tool, so that we can investigate further.",
            err))?);

        let mut sigint = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::interrupt())?;
        let mut sigterm =
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?;
        let mut sigquit = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::quit())?;
        let mut sighup = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::hangup())?;

        loop {
            let sigint = sigint.recv().fuse();
            let sigterm = sigterm.recv().fuse();
            let sigquit = sigquit.recv().fuse();
            let sighup = sighup.recv().fuse();

            pin_mut!(sigint, sigterm, sigquit);

            tokio::select! {
                _ = sigint => {
                    debug!("Forwarding SIGINT to child process.");
                    nix::sys::signal::kill(pid, nix::sys::signal::Signal::SIGINT)?;
                },
                _ = sigterm => {
                    debug!("Forwarding SIGTERM to child process.");
                    nix::sys::signal::kill(pid, nix::sys::signal::Signal::SIGTERM)?;
                },
                _ = sigquit => {
                    debug!("Forwarding SIGQUIT to child process.");
                    nix::sys::signal::kill(pid, nix::sys::signal::Signal::SIGQUIT)?;
                },
                _ = sighup => {
                    debug!("Forwarding SIGHUP to child process.");
                    nix::sys::signal::kill(pid, nix::sys::signal::Signal::SIGHUP)?;
                },
                status = child.wait() => {
                    return Ok(status?.code().unwrap_or_default())
                }
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
        let launcher = launcher(config);

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
        let launcher = launcher(config);

        let result = launcher.run(&a, &t).await.unwrap();
        assert_eq!(result, 123);
    }
}
