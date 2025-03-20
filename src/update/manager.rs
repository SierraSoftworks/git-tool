use super::*;
use crate::{core::Core, errors};
use itertools::Itertools;
use std::path::{Path, PathBuf};
use tracing_batteries::prelude::*;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

pub struct UpdateManager<S = GitHubSource>
where
    S: Source,
{
    pub target_application: PathBuf,

    pub variant: ReleaseVariant,
    pub source: S,

    pub(self) launcher: Box<dyn cmd::Launcher + Send + Sync>,
    pub(self) filesystem: Box<dyn fs::FileSystem + Send + Sync>,
}

impl<S> UpdateManager<S>
where
    S: Source,
{
    #[cfg(test)]
    pub fn new(target_application: PathBuf) -> Self {
        Self {
            target_application,
            ..Default::default()
        }
    }

    #[cfg(test)]
    pub fn with_mock_launcher<M: FnMut(&mut cmd::MockLauncher)>(self, mut setup: M) -> Self {
        let mut mock = cmd::MockLauncher::new();
        setup(&mut mock);
        Self {
            launcher: Box::new(mock),
            ..self
        }
    }

    #[cfg(test)]
    pub fn with_mock_fs<M: FnMut(&mut fs::MockFileSystem)>(self, mut setup: M) -> Self {
        let mut mock = fs::MockFileSystem::new();
        setup(&mut mock);
        Self {
            filesystem: Box::new(mock),
            ..self
        }
    }

    pub async fn get_releases(&self, core: &Core) -> Result<Vec<Release>, errors::Error> {
        self.source.get_releases(core).await
    }

    #[tracing::instrument(err, skip(self, core))]
    pub async fn update(&self, core: &Core, release: &Release) -> Result<bool, errors::Error> {
        let state = UpdateState {
            target_application: Some(self.target_application.clone()),
            temporary_application: Some(self.filesystem.get_temp_app_path(release)),
            phase: UpdatePhase::Prepare,
        };

        let app = state.temporary_application.clone().ok_or_else(|| errors::system(
            "A temporary application path was not provided and the update cannot proceed (prepare -> replace phase).",
            "Please report this issue to us on GitHub, or try updating manually by downloading the latest release from GitHub yourself."))?;

        let variant = release.get_variant(&self.variant).ok_or_else(|| errors::system(
            &format!("Your operating system and architecture are not supported by {}. Supported platforms include: {}", release.id, release.variants.iter().map(|v| format!("{}_{}", v.platform, v.arch)).format(", ")),
            "Please open an issue on GitHub to request that we cross-compile a release of Git-Tool for your platform."))?;

        {
            info!(
                "Checking whether app binary ({}) is writable by current user.",
                self.target_application.display()
            );
            let permissions = tokio::fs::metadata(self.target_application.clone()).await?;
            if permissions.permissions().readonly() {
                return Err(errors::user(
                    "The application binary is read-only. Please make sure that the application binary is writable by the current user.",
                    {
                        #[cfg(windows)] {
                            "Try running this command in an administrative console (Win+X, A)."
                        }

                        #[cfg(unix)]{
                            "Try running this command as root with `sudo git-tool update`."
                        }
                    }));
            }
        }

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

            debug!("Preparing application file for execution.");
            self.prepare_app_file(&app)?;
        }

        self.resume(&state).await
    }

    #[tracing::instrument(err, skip(self))]
    pub async fn resume(&self, state: &UpdateState) -> Result<bool, errors::Error> {
        match state.phase {
            UpdatePhase::NoUpdate => Ok(false),
            UpdatePhase::Prepare => self.prepare(state).await,
            UpdatePhase::Replace => self.replace(state).await,
            UpdatePhase::Cleanup => self.cleanup(state).await,
        }
    }

    #[tracing::instrument(err, skip(self))]
    async fn prepare(&self, state: &UpdateState) -> Result<bool, errors::Error> {
        let next_state = state.for_phase(UpdatePhase::Replace);
        let update_source = state.temporary_application.clone().ok_or_else(|| errors::system(
            "Could not launch the new application version to continue the update process (prepare -> replace phase).",
            "Please report this issue to us on GitHub, or try updating manually by downloading the latest release from GitHub yourself."))?;

        info!("Launching temporary release binary to perform 'replace' phase of update.");
        self.launch(&update_source, &next_state)?;

        Ok(true)
    }

    #[tracing::instrument(err, skip(self))]
    async fn replace(&self, state: &UpdateState) -> Result<bool, errors::Error> {
        let update_source = state.temporary_application.clone().ok_or_else(|| errors::system(
            "Could not locate the temporary update files needed to complete the update process (replace phase).",
            "Please report this issue to us on GitHub, or try updating manually by downloading the latest release from GitHub yourself."))?;
        let update_target = state.target_application.clone().ok_or_else(|| errors::system(
            "Could not locate the application which was meant to be updated due to an issue loading the update state (replace phase).",
            "Please report this issue to us on GitHub, or try updating manually by downloading the latest release from GitHub yourself."))?;

        info!("Removing the original application binary to avoid conflicts with open handles.");
        self.filesystem.delete_file(&update_target).await?;

        info!("Replacing original application binary with temporary release binary.");
        self.filesystem
            .copy_file(&update_source, &update_target)
            .await?;

        info!("Launching updated application to perform 'cleanup' phase of update.");
        let next_state = state.for_phase(UpdatePhase::Cleanup);
        self.launch(&update_target, &next_state)?;

        Ok(true)
    }

    #[tracing::instrument(err, skip(self))]
    async fn cleanup(&self, state: &UpdateState) -> Result<bool, errors::Error> {
        let update_source = state.temporary_application.clone().ok_or_else(|| errors::system(
            "Could not locate the temporary update files needed to complete the update process (cleanup phase).",
            "Please report this issue to us on GitHub, or try updating manually by downloading the latest release from GitHub yourself."))?;

        info!("Removing temporary update application binary.");
        self.filesystem.delete_file(&update_source).await?;

        Ok(true)
    }

    #[cfg(unix)]
    #[tracing::instrument(err, skip(self, file))]
    fn prepare_app_file(&self, file: &Path) -> Result<(), errors::Error> {
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

        debug!(
            "Updating file permissions to 0775 to ensure we can write to the application binary."
        );
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

    #[tracing::instrument(err, skip(self, app_path))]
    fn launch(&self, app_path: &Path, state: &UpdateState) -> Result<(), errors::Error> {
        self.launcher.launch(app_path, state)
    }
}

impl<S> Default for UpdateManager<S>
where
    S: Source,
{
    fn default() -> Self {
        Self {
            target_application: std::env::current_exe().unwrap_or_default(),
            source: S::default(),
            variant: ReleaseVariant::default(),

            launcher: cmd::default(),
            filesystem: fs::default(),
        }
    }
}

impl<S> std::fmt::Debug for UpdateManager<S>
where
    S: Source,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} ({})", &self.source, &self.variant)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_update() {
        let core = Core::builder()
            .with_default_config()
            .with_mock_http_client(github::mocks::mock_get_releases())
            .build();

        let temp = tempdir().unwrap();

        let app_path = temp.path().join("app").to_owned();
        let temp_app_path = temp.path().join("app-temp").to_owned();

        std::fs::write(&app_path, "Pre-Update").unwrap();

        let manager: UpdateManager<GitHubSource> = UpdateManager::new(app_path.clone())
            .with_mock_launcher(|mock| {
                let temp_app_path = temp_app_path.clone();
                mock.expect_launch()
                    .once()
                    .withf(move |path, state| {
                        path == temp_app_path && state.phase == UpdatePhase::Replace
                    })
                    .returning(|_, _| Ok(()));
            })
            .with_mock_fs(|mock| {
                mock.expect_get_temp_app_path()
                    .once()
                    .return_const(temp_app_path.clone());
            });

        let releases = manager
            .get_releases(&core)
            .await
            .expect("we should receive a release entry");

        let latest_release =
            Release::get_latest(releases.iter()).expect("we should receive a latest release entry");

        let has_update = manager
            .update(&core, latest_release)
            .await
            .expect("the update operation should succeed");

        assert!(has_update, "the update should be applied");
    }

    #[tokio::test]
    async fn test_update_resume() {
        let temp = tempdir().unwrap();

        let app_path = temp.path().join("app").to_owned();
        let temp_app_path = temp.path().join("app-temp").to_owned();

        let manager: UpdateManager<GitHubSource> = UpdateManager::new(app_path.clone())
            .with_mock_launcher(|mock| {
                let app_path = app_path.clone();
                mock.expect_launch()
                    .once()
                    .withf(move |path, state| {
                        path == app_path && state.phase == UpdatePhase::Cleanup
                    })
                    .returning(|_, _| Ok(()));
            })
            .with_mock_fs(|mock| {
                let app_path = app_path.clone();
                let app_path_copy = app_path.clone();
                let temp_app_path = temp_app_path.clone();
                mock.expect_get_temp_app_path().never();
                mock.expect_copy_file()
                    .once()
                    .withf(move |src, dst| src == temp_app_path && dst == app_path_copy)
                    .returning(|_, _| Ok(()));
                mock.expect_delete_file()
                    .once()
                    .withf(move |path| path == app_path)
                    .returning(|_| Ok(()));
            });

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
    }

    #[tokio::test]
    async fn test_update_cleanup() {
        let temp = tempdir().unwrap();

        let app_path = temp.path().join("app").to_owned();
        let temp_app_path = temp.path().join("app-temp").to_owned();

        let manager: UpdateManager<GitHubSource> = UpdateManager::new(app_path.clone())
            .with_mock_launcher(|mock| {
                mock.expect_spawn().never();
            })
            .with_mock_fs(|mock| {
                let temp_app_path = temp_app_path.clone();
                mock.expect_get_temp_app_path().never();
                mock.expect_delete_file()
                    .once()
                    .withf(move |path| path == temp_app_path)
                    .returning(|path| {
                        std::fs::remove_file(path).expect("we should be able to delete the path");
                        Ok(())
                    });
            });

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
