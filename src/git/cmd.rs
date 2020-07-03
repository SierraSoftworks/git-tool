use tokio::process::Command;
use crate::errors;
use std::process::Stdio;

pub async fn git_cmd(cmd: &mut Command) -> Result<String, errors::Error> {
    let child = cmd.stdout(Stdio::piped()).spawn()?;
    let output = child.wait_with_output().await?;

    let output_text = String::from_utf8(output.stdout)?;
    
    if !output.status.success() {
        match output.status.code() {
            Some(code) => Err(errors::system_with_internal(
                &format!("Git exited with a non-zero exit code ({}).", code),
                "Please check the output printed by Git to determine why the command failed and take appropriate action.",
                errors::detailed_message(&output_text))),
            None => Err(errors::system_with_internal(
                "Git exited prematurely because it received an unexpected signal.",
                "Please check the output printed by Git to determine why the command failed and take appropriate action.",
            errors::detailed_message(&output_text)))
        }
    } else {
        Ok(output_text)
    }
}