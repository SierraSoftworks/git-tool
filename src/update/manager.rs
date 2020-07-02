use super::*;
use std::{process::Command, path::{Path, PathBuf}};
use crate::errors;
use itertools::Itertools;
use std::time::Duration;

#[cfg(windows)]
use std::os::windows::process::CommandExt;
#[cfg(windows)]
use windows::*;

pub struct UpdateManager<S = super::github::GitHubSource>
where S: Source {
    pub target_application: PathBuf,
    
    pub variant: ReleaseVariant,
    pub source: S,
}

impl<S> UpdateManager<S>
where S: Source {
    pub async fn get_releases(&self) -> Result<Vec<Release>, errors::Error> {
        self.source.get_releases().await
    }

    pub async fn update(&self, release: &Release) -> Result<bool, errors::Error> {
        let state = UpdateState {
            target_application: Some(self.target_application.clone()),
            temporary_application: Some(self.get_temp_app_path(release)),
            phase: UpdatePhase::Prepare
        };

        let app = state.temporary_application.clone().ok_or(errors::system(
            "A temporary application path was not provided and the update cannot proceed (prepare -> replace phase).",
            "Please report this issue to us on GitHub, or try updating manually by downloading the latest release from GitHub yourself."))?;
        
        let variant = release.get_variant(&self.variant).ok_or(errors::system(
            &format!("Your operating system and architecture are not supported by {}. Supported platforms include: {}", release.id, release.variants.iter().map(|v| format!("{}_{}", v.platform, v.arch)).format(", ")),
            "Please open an issue on GitHub to request that we cross-compile a release of Git-Tool for your platform."))?;

        {
            let mut app_file = std::fs::File::create(&app)?;
            self.source.get_binary(release, variant, &mut app_file).await?;
        }

        self.resume(&state).await
    }

    pub async fn resume(&self, state: &UpdateState) -> Result<bool, errors::Error> {
        match state.phase {
            UpdatePhase::NoUpdate => Ok(false),
            UpdatePhase::Prepare => self.prepare(state).await,
            UpdatePhase::Replace => self.replace(state).await,
            UpdatePhase::Cleanup => self.cleanup(state).await
        }
    }

    async fn prepare(&self, state: &UpdateState) -> Result<bool, errors::Error> {
        let next_state = state.for_phase(UpdatePhase::Replace);
        let update_source = state.temporary_application.clone().ok_or(errors::system(
            "Could not launch the new application version to continue the update process (prepare -> replace phase).",
            "Please report this issue to us on GitHub, or try updating manually by downloading the latest release from GitHub yourself."))?;
        
        self.launch(&update_source, &next_state)?;

        Ok(true)
    }

    async fn replace(&self, state: &UpdateState) -> Result<bool, errors::Error> {
        let update_source = state.temporary_application.clone().ok_or(errors::system(
            "Could not locate the temporary update files needed to complete the update process (replace phase).",
            "Please report this issue to us on GitHub, or try updating manually by downloading the latest release from GitHub yourself."))?;
        let update_target = state.target_application.clone().ok_or(errors::system(
            "Could not locate the application which was meant to be updated due to an issue loading the update state (replace phase).",
            "Please report this issue to us on GitHub, or try updating manually by downloading the latest release from GitHub yourself."))?;

        self.copy_file(&update_source, &update_target).await?;

        let next_state = state.for_phase(UpdatePhase::Cleanup);
        self.launch(&update_target, &next_state)?;

        Ok(true)
    }

    async fn cleanup(&self, state: &UpdateState) -> Result<bool, errors::Error> {
        let update_source = state.temporary_application.clone().ok_or(errors::system(
            "Could not locate the temporary update files needed to complete the update process (cleanup phase).",
            "Please report this issue to us on GitHub, or try updating manually by downloading the latest release from GitHub yourself."))?;

        self.delete_file(&update_source).await?;

        Ok(true)
    }

    fn launch(&self, app_path: &Path, state: &UpdateState) -> Result<(), errors::Error> {
        let state_json = serde_json::to_string(state)?;
        let mut cmd = Command::new(app_path);
        cmd
            .arg("--update-resume-internal")
            .arg(&state_json)
            .arg("update")
            .arg("--state")
            .arg(&state_json);

        #[cfg(windows)]
        cmd.creation_flags(DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP);

        cmd.spawn().map_err(|e| errors::system_with_internal(
            &format!("Could not launch the new application version to continue the update process (_ -> {} phase)", state.phase.to_string()),
            "Please report this issue to us on GitHub, or try updating manually by downloading the latest release from GitHub yourself.",
            e))?;

        Ok(())
    }

    async fn delete_file(&self, path: &Path) -> Result<(), errors::Error> {
        let mut retries = 10;

        while retries >= 0 {
            retries -= 1;

            match tokio::fs::remove_file(path).await {
                Err(e) if retries < 0 => return Err(e.into()),
                Ok(_) => {
                    return Ok(())
                },
                _ => tokio::time::delay_for(Duration::from_millis(500)).await
            }
        }

        Ok(())
    }

    async fn copy_file(&self, from: &Path, to: &Path) -> Result<(), errors::Error> {
        let mut retries = 10;

        while retries > 0 {
            retries -= 1;
            match tokio::fs::copy(from, to).await {
                Err(e) if retries < 0 => return Err(e.into()),
                Ok(_) => {
                    return Ok(())
                },
                _ => tokio::time::delay_for(Duration::from_millis(500)).await
            }
        }

        Ok(())
    }

    fn get_temp_app_path(&self, release: &Release) -> PathBuf {
        let file_name = format!("git-tool-update-{}{}", release.id, self.target_application.extension().and_then(|e| e.to_str()).map(|e| ".".to_string() + e).unwrap_or(if cfg!(windows) { ".exe" } else { "" }.to_string()));
        std::env::temp_dir().join(file_name)
    }
}

impl<S> Default for UpdateManager<S>
where S: Source {
    fn default() -> Self {
        Self {
            target_application: PathBuf::from(std::env::args().nth(0).unwrap_or_default()),
            source: S::default(),
            variant: ReleaseVariant::default()
        }
    }
}

#[cfg(windows)]
mod windows {
    pub const DETACHED_PROCESS: u32 = 0x00000008;
    pub const CREATE_NEW_PROCESS_GROUP: u32 = 0x00000200;
}