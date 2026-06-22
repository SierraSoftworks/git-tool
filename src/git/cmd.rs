use human_errors::ResultExt;
use std::path::Path;
use std::process::Stdio;
use tokio::process::Command;
use tracing_batteries::prelude::*;

/// The status code git returns when it is invoked with an invalid command line
/// (its `usage` exit code). Because Git-Tool is responsible for assembling every
/// git command it runs, a usage error reflects a bug in Git-Tool rather than
/// something the user did wrong, so we keep reporting these as system errors.
const GIT_USAGE_EXIT_CODE: i32 = 129;

#[tracing::instrument(err, skip(cmd), fields(otel.kind = ?opentelemetry::trace::SpanKind::Client, command = EmptyField, otel.status_code = 0, status_code = EmptyField))]
pub async fn git_cmd(cmd: &mut Command) -> Result<String, human_errors::Error> {
    // Record a redacted representation of the command on the current span. We
    // deliberately avoid recording the raw command (`{cmd:?}`), because its
    // debug rendering includes the working directory and the full argument
    // list, both of which can contain user-provided information (repository
    // URLs, branch names, filesystem paths, commit messages) that we don't want
    // to leak to our telemetry backends.
    let command = redact_command(cmd);
    Span::current().record("command", command.as_str());

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
                Err(git_exit_error(&command, code, &output_text))
            }
            None => {
                Span::current()
                    .record("status_code", 1_i32)
                    .record("otel.status_code", 2_u32);
                // A missing exit code means git was terminated by a signal (for
                // example the user pressing Ctrl+C, or the OS killing the
                // process). This isn't a Git-Tool bug, so we surface it as a
                // user error rather than reporting it to Sentry.
                Err(human_errors::wrap_user(
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

/// Builds the appropriate [`human_errors::Error`] for a git command that exited
/// with a non-zero status code.
///
/// Git command failures are overwhelmingly caused by the state of the user's
/// repository or their input (merge conflicts, missing branches, authentication
/// failures, and so on) rather than by a bug in Git-Tool, so we classify them as
/// user errors and keep them out of Sentry. The exception is git's "usage" exit
/// code ([`GIT_USAGE_EXIT_CODE`]), which means we handed git an invalid command
/// line — a genuine Git-Tool bug that we do want to hear about, so it remains a
/// system error.
fn git_exit_error(command: &str, code: i32, output: &str) -> human_errors::Error {
    if code == GIT_USAGE_EXIT_CODE {
        human_errors::system(
            format!("{command} exited with status code {code}. {output}"),
            &[
                "This looks like a bug in Git-Tool itself. Please report it at https://github.com/SierraSoftworks/git-tool/issues so that we can fix it.",
            ],
        )
    } else {
        human_errors::user(
            format!("{command} exited with status code {code}. {output}"),
            &[
                "Please check the output printed by Git to determine why the command failed and take appropriate action.",
            ],
        )
    }
}

/// Produces a redacted, human-readable rendering of a git command that is safe
/// to include in telemetry and error reports.
///
/// The git sub-command and any flags are preserved, because they are literals
/// baked into Git-Tool and are invaluable when debugging (a usage error, for
/// instance, points straight at the offending flag). Every other argument is
/// replaced with a `<redacted>` placeholder, since those positional arguments
/// carry user-provided values — repository URLs, branch names, filesystem paths,
/// commit messages, and the like — that we must not leak.
fn redact_command(cmd: &Command) -> String {
    let mut rendered = String::from("git");
    let mut subcommand_seen = false;

    for arg in cmd.as_std().get_args() {
        let arg = arg.to_string_lossy();

        rendered.push(' ');
        if arg.starts_with('-') {
            // Flags (including inline `--key=value` literals such as
            // `--format=...`) never carry user-provided data in Git-Tool, so
            // they are safe to surface.
            rendered.push_str(&arg);
        } else if !subcommand_seen {
            // The first positional argument is the git sub-command (`clone`,
            // `switch`, ...), which is likewise a literal and helpful to retain.
            subcommand_seen = true;
            rendered.push_str(&arg);
        } else {
            rendered.push_str("<redacted>");
        }
    }

    rendered
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::git_init;
    use tempfile::tempdir;

    #[test]
    fn redact_command_keeps_subcommand_and_flags() {
        let mut cmd = Command::new("git");
        cmd.arg("clone")
            .arg("--recurse-submodules")
            .arg("https://github.com/sierrasoftworks/private-repo.git")
            .arg("/home/user/dev/github.com/sierrasoftworks/private-repo");

        assert_eq!(
            redact_command(&cmd),
            "git clone --recurse-submodules <redacted> <redacted>"
        );
    }

    #[test]
    fn redact_command_redacts_flag_values() {
        // `-m <message>` passes the (potentially sensitive) commit message as a
        // separate argument, so it must be redacted while the `-m` flag itself
        // remains visible.
        let mut cmd = Command::new("git");
        cmd.arg("commit")
            .arg("-m")
            .arg("Fix the super secret feature")
            .arg("secret-file.txt");

        assert_eq!(redact_command(&cmd), "git commit -m <redacted> <redacted>");
    }

    #[test]
    fn redact_command_keeps_inline_flag_literals() {
        let mut cmd = Command::new("git");
        cmd.arg("branch")
            .arg("-a")
            .arg("--format=%(refname:lstrip=2)");

        assert_eq!(
            redact_command(&cmd),
            "git branch -a --format=%(refname:lstrip=2)"
        );
    }

    #[test]
    fn git_exit_error_usage_code_is_system() {
        let err = git_exit_error("git status", GIT_USAGE_EXIT_CODE, "");
        assert!(
            err.is(human_errors::Kind::System),
            "git usage errors should be reported as system errors so that we hear about them"
        );
    }

    #[test]
    fn git_exit_error_other_codes_are_user() {
        for code in [1, 2, 128] {
            let err = git_exit_error("git switch", code, "error: something went wrong");
            assert!(
                err.is(human_errors::Kind::User),
                "exit code {code} should be classified as a user error"
            );
            assert!(
                !err.is(human_errors::Kind::System),
                "exit code {code} should not be reported to Sentry as a system error"
            );
        }
    }

    #[tokio::test]
    async fn git_cmd_normal_failure_is_redacted_user_error() {
        let temp = tempdir().unwrap();
        git_init(temp.path()).await.unwrap();

        // Checking out a branch that doesn't exist makes git exit with a normal
        // (non-usage) error code, which we treat as a user error. The branch
        // name the user provided must not appear in the resulting error.
        let secret_branch = "super-secret-branch-name";
        let err = git_cmd(
            Command::new("git")
                .current_dir(temp.path())
                .arg("checkout")
                .arg(secret_branch),
        )
        .await
        .expect_err("checking out a missing branch should fail");

        assert!(
            err.is(human_errors::Kind::User),
            "a normal git failure should be a user error: {err}"
        );
        assert!(
            !err.is(human_errors::Kind::System),
            "a normal git failure should not be reported to Sentry"
        );

        let rendered = format!("{err}");
        assert!(
            !rendered.contains(secret_branch),
            "the user-provided branch name should be redacted from the error: {rendered}"
        );
        assert!(
            rendered.contains("<redacted>"),
            "the error should reference the redaction placeholder: {rendered}"
        );
    }

    #[tokio::test]
    async fn git_cmd_usage_failure_is_system_error() {
        let temp = tempdir().unwrap();
        git_init(temp.path()).await.unwrap();

        // An unknown flag makes git exit with its usage status code (129),
        // signalling that Git-Tool built an invalid command line.
        let err = git_cmd(
            Command::new("git")
                .current_dir(temp.path())
                .arg("status")
                .arg("--this-is-not-a-real-flag"),
        )
        .await
        .expect_err("an unknown flag should make git fail");

        assert!(
            err.is(human_errors::Kind::System),
            "a git usage error should be reported as a system error: {err}"
        );
    }
}
