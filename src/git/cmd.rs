use crate::errors;
use opentelemetry::trace::{SpanKind, StatusCode};
use tracing::{field, Span};
use std::process::Stdio;
use tokio::process::Command;

#[tracing::instrument(err, skip(cmd), fields(otel.kind = %SpanKind::Client, command=?cmd, otel.status = ?StatusCode::Ok, status_code = field::Empty))]
pub async fn git_cmd(cmd: &mut Command) -> Result<String, errors::Error> {
    // NOTE: We disable logging to stdout to avoid breaking the test output
    #[cfg(test)]
    let cmd = cmd.stderr(Stdio::piped());

    let child = cmd.stdout(Stdio::piped()).spawn().map_err(|err| errors::user_with_internal(
        "Could not run git, which is a dependency of Git-Tool.",
        "Please ensure that git is installed, present on your $PATH and executable before running Git-Tool again.",
        err
    ))?;
    let output = child.wait_with_output().await.map_err(|err| errors::user_with_internal(
        "Could not run git, which is a dependency of Git-Tool.",
        "Please ensure that git is installed, present on your $PATH and executable before running Git-Tool again.",
        err
    ))?;

    let output_text = String::from_utf8(output.stdout)?;

    if !output.status.success() {
        match output.status.code() {
            Some(code) => {
                Span::current()
                    .record("status_code", &code)
                    .record("otel.status", &field::debug(StatusCode::Error));
                Err(errors::user_with_cause(
                    "Git exited with a failure status code.",
                    "Please check the output printed by Git to determine why the command failed and take appropriate action.",
                    errors::system(&format!("{:?} exited with status code {}.", cmd, code), &output_text)))
            },
            None => {
                Span::current().record("status_code", &1_i32)
                    .record("otel.status", &field::debug(StatusCode::Error));
                Err(errors::system_with_internal(
                    "Git exited prematurely because it received an unexpected signal.",
                    "Please check the output printed by Git to determine why the command failed and take appropriate action.",
                    errors::detailed_message(&output_text)))
            }
        }
    } else {
        Ok(output_text)
    }
}
