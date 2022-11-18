use opentelemetry::propagation::TextMapPropagator;
use std::{path::Path, process::Command};
use tracing_opentelemetry::OpenTelemetrySpanExt;

#[cfg(windows)]
use std::os::windows::process::CommandExt;
#[cfg(windows)]
use windows::*;

#[cfg(test)]
use mockall::automock;

use crate::errors;

use super::UpdateState;

pub(super) fn default() -> Box<dyn Launcher + Send + Sync> {
    Box::new(DefaultLauncher {})
}

#[cfg_attr(test, automock)]
pub trait Launcher {
    fn launch(&self, app_path: &Path, state: &UpdateState) -> Result<(), errors::Error> {
        let trace_context = {
            let mut context = std::collections::HashMap::new();
            let propagator = opentelemetry::sdk::propagation::TraceContextPropagator::new();
            propagator.inject_context(&tracing::Span::current().context(), &mut context);

            serde_json::to_string(&context)
        }?;

        let state_json = serde_json::to_string(state)?;

        let mut cmd = Command::new(app_path);
        cmd.arg("--update-resume-internal")
            .arg(&state_json)
            .arg("--trace-context")
            .arg(&trace_context)
            .arg("update")
            .arg("--state")
            .arg(&state_json);

        #[cfg(windows)]
        cmd.creation_flags(DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP);

        self.spawn(cmd).map_err(|e| errors::system_with_internal(
            &format!("Could not launch the new application version to continue the update process (_ -> {} phase)", state.phase.to_string()),
            "Please report this issue to us on GitHub, or try updating manually by downloading the latest release from GitHub yourself.",
            e))
    }

    fn spawn(&self, cmd: Command) -> Result<(), errors::Error>;
}

struct DefaultLauncher {}

impl Launcher for DefaultLauncher {
    fn spawn(&self, mut cmd: Command) -> Result<(), errors::Error> {
        cmd.spawn()?;

        Ok(())
    }
}
