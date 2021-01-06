use crate::errors;
use std::process::Stdio;
use tokio::process::Command;

pub async fn git_cmd(cmd: &mut Command) -> Result<String, errors::Error> {
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
            Some(code) => Err(errors::system_with_cause(
                "Git exited with a failure status code.",
                "Please check the output printed by Git to determine why the command failed and take appropriate action.",
                errors::system(&format!("{:?} exited with status code {}.", cmd, code), &output_text))),
            None => Err(errors::system_with_internal(
                "Git exited prematurely because it received an unexpected signal.",
                "Please check the output printed by Git to determine why the command failed and take appropriate action.",
            errors::detailed_message(&output_text)))
        }
    } else {
        Ok(output_text)
    }
}
