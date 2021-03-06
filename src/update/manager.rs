use super::*;
use crate::{core::Core, errors};
use itertools::Itertools;
use std::time::Duration;
use std::{
    path::{Path, PathBuf},
    process::Command,
};

#[cfg(windows)]
use std::os::windows::process::CommandExt;
#[cfg(windows)]
use windows::*;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

#[cfg(test)]
use mocktopus::macros::*;

pub struct UpdateManager<S = super::github::GitHubSource>
where
    S: Source,
{
    pub target_application: PathBuf,

    pub variant: ReleaseVariant,
    pub source: S,
}

impl<S> UpdateManager<S>
where
    S: Source,
{
    pub async fn get_releases(&self, core: &Core) -> Result<Vec<Release>, errors::Error> {
        self.source.get_releases(core).await
    }

    pub async fn update(&self, core: &Core, release: &Release) -> Result<bool, errors::Error> {
        let state = UpdateState {
            target_application: Some(self.target_application.clone()),
            temporary_application: Some(self.get_temp_app_path(release)),
            phase: UpdatePhase::Prepare,
        };

        let app = state.temporary_application.clone().ok_or(errors::system(
            "A temporary application path was not provided and the update cannot proceed (prepare -> replace phase).",
            "Please report this issue to us on GitHub, or try updating manually by downloading the latest release from GitHub yourself."))?;

        let variant = release.get_variant(&self.variant).ok_or(errors::system(
            &format!("Your operating system and architecture are not supported by {}. Supported platforms include: {}", release.id, release.variants.iter().map(|v| format!("{}_{}", v.platform, v.arch)).format(", ")),
            "Please open an issue on GitHub to request that we cross-compile a release of Git-Tool for your platform."))?;

        {
            info!(
                "Downloading release binary for {} to temporary location ({}).",
                release.version,
                app.display()
            );
            let mut app_file = std::fs::File::create(&app).map_err(|err| {
                errors::user_with_internal(
                    &format!(
                        "Could not create the new application file '{}' due to an OS-level error.",
                        app.display()
                    ),
                    "Check that Git-Tool has permission to create and write to this file and that the parent directory exists.",
                    err,
                )
            })?;
            self.source
                .get_binary(core, release, variant, &mut app_file)
                .await?;

            self.prepare_app_file(&app)?;
        }

        self.resume(&state).await
    }

    pub async fn resume(&self, state: &UpdateState) -> Result<bool, errors::Error> {
        match state.phase {
            UpdatePhase::NoUpdate => Ok(false),
            UpdatePhase::Prepare => self.prepare(state).await,
            UpdatePhase::Replace => self.replace(state).await,
            UpdatePhase::Cleanup => self.cleanup(state).await,
        }
    }

    async fn prepare(&self, state: &UpdateState) -> Result<bool, errors::Error> {
        let next_state = state.for_phase(UpdatePhase::Replace);
        let update_source = state.temporary_application.clone().ok_or(errors::system(
            "Could not launch the new application version to continue the update process (prepare -> replace phase).",
            "Please report this issue to us on GitHub, or try updating manually by downloading the latest release from GitHub yourself."))?;

        info!("Launching temporary release binary to perform 'replace' phase of update.");
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

        info!("Replacing original application binary with temporary release binary.");
        self.copy_file(&update_source, &update_target).await?;

        info!("Launching updated application to perform 'cleanup' phase of update.");
        let next_state = state.for_phase(UpdatePhase::Cleanup);
        self.launch(&update_target, &next_state)?;

        Ok(true)
    }

    async fn cleanup(&self, state: &UpdateState) -> Result<bool, errors::Error> {
        let update_source = state.temporary_application.clone().ok_or(errors::system(
            "Could not locate the temporary update files needed to complete the update process (cleanup phase).",
            "Please report this issue to us on GitHub, or try updating manually by downloading the latest release from GitHub yourself."))?;

        info!("Removing temporary update application binary.");
        self.delete_file(&update_source).await?;

        Ok(true)
    }

    #[cfg(unix)]
    fn prepare_app_file(&self, file: &std::path::Path) -> Result<(), errors::Error> {
        let mut perms = std::fs::metadata(file).map_err(|err| {
            errors::user_with_internal(
                &format!(
                    "Could not gather permissions information for '{}' due to an OS-level error.",
                    file.display()
                ),
                "Check that Git-Tool has permission to read this file and that the parent directory exists.",
                err,
            )
        })?.permissions();

        // u=rwx,g=rwx,o=rx
        perms.set_mode(0o775);
        std::fs::set_permissions(file, perms).map_err(|err| {
            errors::user_with_internal(
                &format!(
                    "Could not set executable permissions on '{}' due to an OS-level error.",
                    file.display()
                ),
                "Check that Git-Tool has permission to modify permissions for this file and that the parent directory exists.",
                err,
            )
        })?;

        Ok(())
    }

    #[cfg(not(unix))]
    fn prepare_app_file(&self, _file: &std::path::Path) -> Result<(), errors::Error> {
        Ok(())
    }
}

#[cfg_attr(test, mockable)]
impl<S: Source> UpdateManager<S> {
    fn launch(&self, app_path: &Path, state: &UpdateState) -> Result<(), errors::Error> {
        let state_json = serde_json::to_string(state)?;
        let mut cmd = Command::new(app_path);
        cmd.arg("--update-resume-internal")
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
        let max_retries = 10;
        let mut retries = max_retries;

        while retries >= 0 {
            retries -= 1;

            match tokio::fs::remove_file(path).await {
                Err(e) if retries < 0 => return Err(errors::user_with_internal(
                    &format!("Could not remove the old application file '{}' after {} retries.", path.display(), max_retries),
                    "This probably means that Git-Tool is still running in another terminal. Please exit any running Git-Tool processes (including shells launched by it) before trying again.",
                    e
                )),
                Ok(_) => return Ok(()),
                _ => tokio::time::sleep(Duration::from_millis(500)).await,
            }
        }

        Ok(())
    }

    async fn copy_file(&self, from: &Path, to: &Path) -> Result<(), errors::Error> {
        let max_retries = 10;
        let mut retries = max_retries;

        while retries > 0 {
            retries -= 1;
            match tokio::fs::copy(from, to).await {
                Err(e) if retries < 0 => return Err(errors::user_with_internal(
                    &format!("Could not copy the new application file '{}' to overwrite the old application file '{}' after {} retries.", from.display(), to.display(), max_retries),
                    "This probably means that Git-Tool is still running in another terminal. Please exit any running Git-Tool processes (including shells launched by it) before trying again.",
                    e
                )),
                Ok(_) => return Ok(()),
                _ => tokio::time::sleep(Duration::from_millis(500)).await,
            }
        }

        Ok(())
    }

    fn get_temp_app_path(&self, release: &Release) -> PathBuf {
        let file_name = format!(
            "git-tool-update-{}{}",
            release.id,
            self.target_application
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| ".".to_string() + e)
                .unwrap_or(if cfg!(windows) { ".exe" } else { "" }.to_string())
        );
        std::env::temp_dir().join(file_name)
    }
}

impl<S> Default for UpdateManager<S>
where
    S: Source,
{
    fn default() -> Self {
        Self {
            target_application: PathBuf::from(std::env::args().nth(0).unwrap_or_default()),
            source: S::default(),
            variant: ReleaseVariant::default(),
        }
    }
}

#[cfg(windows)]
mod windows {
    pub const DETACHED_PROCESS: u32 = 0x00000008;
    pub const CREATE_NEW_PROCESS_GROUP: u32 = 0x00000200;
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use super::*;
    use mocktopus::mocking::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_update() {
        let core = Core::builder().build();
        let temp = tempdir().unwrap();

        let app_path = temp.path().join("app").to_owned();
        let temp_app_path = temp.path().join("app-temp").to_owned();

        let mut manager: UpdateManager<GitHubSource> = UpdateManager::default();
        manager.target_application = app_path.clone();

        let launched = Arc::new(Mutex::new(false));

        github::mocks::mock_get_releases();

        {
            let launched = launched.clone();
            let temp_app_path = temp_app_path.clone();
            UpdateManager::<GitHubSource>::launch.mock_safe(move |_, app, state| {
                assert_eq!(app, temp_app_path, "it should launch the temporary app");

                assert!(app.exists(), "the launched app should exist");

                assert_eq!(
                    state.phase,
                    UpdatePhase::Replace,
                    "it should launch the temp app in replace mode"
                );

                assert_eq!(
                    app_path,
                    state.target_application.clone().unwrap().as_path(),
                    "it should pass the correct app path to the temporary app"
                );

                assert_eq!(
                    state.temporary_application.clone().unwrap(),
                    temp_app_path,
                    "it should pass the correct temp app path to the temporary app"
                );

                launched
                    .lock()
                    .map(|mut v| *v = true)
                    .expect("we should be able to set launched to true");

                MockResult::Return(Ok(()))
            });
        }

        {
            let temp_app_path = temp_app_path.clone();
            UpdateManager::<GitHubSource>::get_temp_app_path
                .mock_safe(move |_, _release| MockResult::Return(temp_app_path.clone()));
        }

        let releases = manager
            .get_releases(&core)
            .await
            .expect("we should receive a release entry");

        let latest_release =
            Release::get_latest(releases.iter()).expect("we should receive a latest release entry");

        let has_update = manager
            .update(&core, &latest_release)
            .await
            .expect("the update operation should succeed");

        assert!(has_update, "the update should be applied");
        assert!(
            launched.lock().map(|v| *v).unwrap(),
            "the temporary app should have been launched"
        );
    }

    #[tokio::test]
    async fn test_update_resume() {
        let temp = tempdir().unwrap();

        let app_path = temp.path().join("app").to_owned();
        let temp_app_path = temp.path().join("app-temp").to_owned();

        let mut manager: UpdateManager<GitHubSource> = UpdateManager::default();
        manager.target_application = app_path.clone();

        let launched = Arc::new(Mutex::new(false));

        {
            let launched = launched.clone();
            let app_path = app_path.clone();
            let temp_app_path = temp_app_path.clone();
            UpdateManager::<GitHubSource>::launch.mock_safe(move |_, app, state| {
                assert_eq!(app, app_path, "it should launch the updated app");

                assert!(app.exists(), "the launched app should exist");

                assert_eq!(
                    std::fs::read_to_string(app)
                        .expect("we should be able to read the contents of the app"),
                    "new",
                    "the app binary should have been replaced with the new binary"
                );

                assert_eq!(
                    state.phase,
                    UpdatePhase::Cleanup,
                    "it should launch the app in cleanup mode"
                );

                assert_eq!(
                    app_path,
                    state.target_application.clone().unwrap().as_path(),
                    "it should pass the correct app path to the temporary app"
                );

                assert_eq!(
                    state.temporary_application.clone().unwrap(),
                    temp_app_path,
                    "it should pass the correct temp app path to the temporary app"
                );

                launched
                    .lock()
                    .map(|mut v| *v = true)
                    .expect("we should be able to set launched to true");

                MockResult::Return(Ok(()))
            });
        }

        {
            let temp_app_path = temp_app_path.clone();
            UpdateManager::<GitHubSource>::get_temp_app_path
                .mock_safe(move |_, _release| MockResult::Return(temp_app_path.clone()));
        }

        {
            std::fs::write(&app_path, "original")
                .expect("we should be able to write a payload to the app path");
            std::fs::write(&temp_app_path, "new")
                .expect("we should be able to write a payload to the temp app path");
        }

        let state = UpdateState {
            phase: UpdatePhase::Replace,
            target_application: Some(app_path.clone()),
            temporary_application: Some(temp_app_path.clone()),
        };

        let has_update = manager
            .resume(&state)
            .await
            .expect("the update operation should succeed");

        assert!(has_update, "the update should be applied");
        assert!(
            launched.lock().map(|v| *v).unwrap(),
            "the temporary app should have been launched"
        );
    }

    #[tokio::test]
    async fn test_update_cleanup() {
        let temp = tempdir().unwrap();

        let app_path = temp.path().join("app").to_owned();
        let temp_app_path = temp.path().join("app-temp").to_owned();

        let mut manager: UpdateManager<GitHubSource> = UpdateManager::default();
        manager.target_application = app_path.clone();

        {
            UpdateManager::<GitHubSource>::launch.mock_safe(move |_, _app, _state| {
                panic!("It should not launch an app");
            });
        }

        {
            let temp_app_path = temp_app_path.clone();
            UpdateManager::<GitHubSource>::get_temp_app_path
                .mock_safe(move |_, _release| MockResult::Return(temp_app_path.clone()));
        }

        {
            std::fs::write(&app_path, "original")
                .expect("we should be able to write a payload to the app path");
            std::fs::write(&temp_app_path, "new")
                .expect("we should be able to write a payload to the temp app path");
        }

        let state = UpdateState {
            phase: UpdatePhase::Cleanup,
            target_application: Some(app_path.clone()),
            temporary_application: Some(temp_app_path.clone()),
        };

        let has_update = manager
            .resume(&state)
            .await
            .expect("the update operation should succeed");

        assert!(has_update, "the update should be applied");
        assert!(app_path.exists(), "the app should still be present");
        assert!(
            !temp_app_path.exists(),
            "the temp app should have been removed"
        );
    }
}
