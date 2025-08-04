use crate::errors;
use std::path::Path;
use std::process::Stdio;
use tokio::process::Command;
use tracing_batteries::prelude::*;

#[tracing::instrument(err, skip(cmd), fields(otel.kind = ?opentelemetry::trace::SpanKind::Client, command=?cmd, otel.status_code = 0, status_code = EmptyField))]
pub async fn git_cmd(cmd: &mut Command) -> Result<String, errors::Error> {
    // NOTE: We disable logging to stdout to avoid breaking the test output
    #[cfg(test)]
    let cmd = cmd.stderr(Stdio::piped());

    let child = cmd.stdout(Stdio::piped()).spawn().map_err(|err| errors::user_with_internal(
        "Failed to run git, which is a dependency of Git-Tool.",
        "Please ensure that git is installed, available in your $PATH, and that Git-Tool has permission to execute it. Also check that the folder you are running git in exists, and that git has permission to access it.",
        err
    ))?;

    let output = child.wait_with_output().await.map_err(|err| {
        errors::user_with_internal(
            "Git was started, but Git-Tool failed to retrieve its output.",
            "This may indicate an issue with system resources or git crashing during execution.",
            err,
        )
    })?;

    let output_text = String::from_utf8(output.stdout)?;

    if !output.status.success() {
        match output.status.code() {
            Some(code) => {
                Span::current()
                    .record("status_code", code)
                    .record("otel.status_code", 2_u32);
                Err(errors::user_with_cause(
                    "Git exited with a failure status code.",
                    "Please check the output printed by Git to determine why the command failed and take appropriate action.",
                    errors::system(
                        &format!("{cmd:?} exited with status code {code}."),
                        &output_text,
                    ),
                ))
            }
            None => {
                Span::current()
                    .record("status_code", 1_i32)
                    .record("otel.status_code", 2_u32);
                Err(errors::system_with_internal(
                    "Git exited prematurely because it received an unexpected signal.",
                    "Please check the output printed by Git to determine why the command failed and take appropriate action.",
                    errors::detailed_message(&output_text),
                ))
            }
        }
    } else {
        Ok(output_text)
    }
}

pub fn validate_repo_path_exists(repo: &Path) -> Result<(), errors::Error> {
    if !repo.exists() {
        Err(errors::user(
            &format!("The repository path '{}' does not exist.", repo.display()),
            "Please check that the path is correct and that you have permission to access it.",
        ))
    } else {
        Ok(())
    }
}
