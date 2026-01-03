use human_errors::ResultExt;
use std::path::Path;
use std::process::Stdio;
use tokio::process::Command;
use tracing_batteries::prelude::*;

#[tracing::instrument(err, skip(cmd), fields(otel.kind = ?opentelemetry::trace::SpanKind::Client, command=?cmd, otel.status_code = 0, status_code = EmptyField))]
pub async fn git_cmd(cmd: &mut Command) -> Result<String, human_errors::Error> {
    // NOTE: We disable logging to stdout to avoid breaking the test output
    #[cfg(test)]
    let cmd = cmd.stderr(Stdio::piped());

    let child = cmd.stdout(Stdio::piped()).spawn().wrap_user_err(
        "Failed to run git, which is a dependency of Git-Tool.",
        &["Please ensure that git is installed, available in your $PATH, and that Git-Tool has permission to execute it. Also check that the folder you are running git in exists, and that git has permission to access it."],
    )?;

    let output = child.wait_with_output().await.wrap_user_err(
        "Git was started, but Git-Tool failed to retrieve its output.",
        &["This may indicate an issue with system resources or git crashing during execution."],
    )?;
    let output_text = String::from_utf8(output.stdout).wrap_user_err(
        "Could not parse the output of your Git command as valid UTF-8 text.",
        &["Please ensure that your system is functioning correctly and that git is not producing invalid output."],
    )?;

    if !output.status.success() {
        match output.status.code() {
            Some(code) => {
                Span::current()
                    .record("status_code", code)
                    .record("otel.status_code", 2_u32);
                Err(human_errors::system(
                    format!("{cmd:?} exited with status code {code}. {output_text}"),
                    &[
                        "Please check the output printed by Git to determine why the command failed and take appropriate action.",
                    ],
                ))
            }
            None => {
                Span::current()
                    .record("status_code", 1_i32)
                    .record("otel.status_code", 2_u32);
                Err(human_errors::wrap_system(
                    output_text,
                    "Git exited prematurely because it received an unexpected signal.",
                    &[
                        "Please check the output printed by Git to determine why the command failed and take appropriate action.",
                    ],
                ))
            }
        }
    } else {
        Ok(output_text)
    }
}

pub fn validate_repo_path_exists(repo: &Path) -> Result<(), human_errors::Error> {
    if !repo.exists() {
        Err(human_errors::user(
            format!("The repository path '{}' does not exist.", repo.display()),
            &["Please check that the path is correct and that you have permission to access it."],
        ))
    } else {
        Ok(())
    }
}
