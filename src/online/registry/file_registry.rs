use super::*;
use crate::{errors, fs::to_native_path};
use std::{fs::read_dir, fs::read_to_string, path::PathBuf};

pub struct FileRegistry {
    path: PathBuf,
}

impl FileRegistry {
    #[allow(dead_code)]
    fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

#[async_trait::async_trait]
impl Registry for FileRegistry {
    #[tracing::instrument(err, ret, skip(self, _core))]
    async fn get_entries(&self, _core: &Core) -> Result<Vec<String>, Error> {
        let mut entries = Vec::new();

        for entry in read_dir(&self.path).map_err(|err| {
            errors::user_with_internal(
                &format!(
                    "Could not enumerate the directories within the local filesystem registry folder '{}' due to an OS-level error.",
                    self.path.display()
                ),
                "Check that the directory exists and that Git-Tool has read access to it.",
                err,
            )
        })? {
            let type_entry = entry.map_err(|err| errors::user_with_internal(
                &format!("Could not enumerate the directories within the local filesystem registry folder '{}' due to an OS-level error.", self.path.display()),
                "Check that the directory exists and that Git-Tool has read access to it and its children.",
                err
            ))?;

            if !type_entry.path().is_dir() {
                continue;
            }

            for entry in read_dir(type_entry.path()).map_err(|err| {
                errors::user_with_internal(
                    &format!(
                        "Could not enumerate the files within the local filesystem registry folder '{}' due to an OS-level error.",
                        type_entry.path().display()
                    ),
                    "Check that the directory exists and that Git-Tool has read access to it.",
                    err,
                )
            })? {
                let file_entry =
                    entry.map_err(|err| {
                        errors::user_with_internal(
                            &format!(
                                "Could not enumerate the files within the local filesystem registry folder '{}' due to an OS-level error.",
                                type_entry.path().display()
                            ),
                            "Check that the directory exists and that Git-Tool has read access to it and the files within.",
                            err,
                        )
                    })?;

                if file_entry
                    .file_name()
                    .to_str()
                    .map(|s| s.ends_with(".yaml"))
                    .unwrap_or_default()
                {
                    if let Some(file_name) = PathBuf::from(file_entry.file_name()).file_stem() {
                        entries.push(format!(
                            "{}/{}",
                            PathBuf::from(type_entry.file_name()).display(),
                            PathBuf::from(file_name).display()
                        ));
                    }
                }
            }
        }

        Ok(entries)
    }

    #[tracing::instrument(err, ret, skip(self, _core))]
    async fn get_entry(&self, _core: &Core, id: &str) -> Result<Entry, Error> {
        let path = self.path.join(to_native_path(format!("{}.yaml", id)));
        let contents = read_to_string(&path).map_err(|err| {
            errors::user_with_internal(
                &format!(
                    "Could not read the local filesystem registry entry '{}' due to an OS-level error.",
                    path.display()
                ),
                "Check that the file exists and that Git-Tool has read access to it.",
                err,
            )
        })?;

        Ok(serde_yaml::from_str(&contents).map_err(|err| {
            errors::user_with_internal(
                &format!(
                    "Could not deserialize the registry entry '{}' due to a YAML parser error.",
                    id
                ),
                "Check that the registry entry is valid YAML and matches the registry entry schema.",
                err,
            )
        })?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::get_repo_root;

    #[tokio::test]
    async fn get_entries() {
        let registry = FileRegistry::new(get_repo_root().join("registry"));
        let core = Core::builder().build();

        let entries = registry.get_entries(&core).await.unwrap();
        assert_ne!(entries.len(), 0);
        assert!(entries.iter().any(|i| i == "apps/bash"));
    }

    #[tokio::test]
    async fn get_entry() {
        let registry = FileRegistry::new(get_repo_root().join("registry"));
        let core = Core::builder().build();

        let entry = registry.get_entry(&core, "apps/bash").await.unwrap();
        assert_eq!(entry.name, "Bash");
    }
}

#[cfg(test)]
mod registry_compliance {
    use super::*;
    use crate::test::get_repo_root;

    #[tokio::test]
    async fn registry_validation() {
        let registry = FileRegistry::new(get_repo_root().join("registry"));
        let core = Core::builder().build();

        let mut valid = true;
        for entry in registry.get_entries(&core).await.unwrap() {
            println!("Validating {}", entry);
            match validate_entry(&registry, &entry).await {
                Ok(v) => {
                    valid = valid && v;
                }
                Err(e) => {
                    println!("{}", e.message());
                    valid = false;
                }
            }
        }

        assert!(valid, "all registry entries should be valid");
    }

    async fn validate_entry(registry: &FileRegistry, name: &str) -> Result<bool, Error> {
        let core = Core::builder().build();
        let entry = registry.get_entry(&core, name).await?;
        let mut valid = true;

        let is_app = name.starts_with("apps/");
        let is_service = name.starts_with("services/");

        if !name.is_ascii() {
            println!("- {} has a non-ascii ID", name);
            valid = false;
        }

        if entry.name.is_empty() {
            println!("- {} has an empty name field", name);
            valid = false;
        }

        if entry.description.is_empty() {
            println!("- {} has an empty description field", name);
            valid = false;
        }

        let test_repo = Repo::new(
            "example.com:test/repo",
            PathBuf::from("/dev/example.com/test/repo"),
        );

        for config in entry.configs {
            if config.platform.is_empty() {
                println!(
                    "- {} has a config which is missing the platform field",
                    name
                );
                valid = false;
            }

            if is_app && config.app.is_none() {
                println!("- {} is in the apps/ namespace but has a configuration which is missing an app setting", name);
                valid = false;
            }

            if is_service && config.service.is_none() {
                println!("- {} is in the services/ namespace but has a configuration which is missing a service setting", name);
                valid = false;
            }

            if let Some(app) = config.app {
                if app.name.is_empty() {
                    println!(
                        "- {}#{} has an app entry which is missing its name",
                        name, &config.platform
                    );
                    valid = false;
                }

                if app.command.is_empty() {
                    println!(
                        "- {}#{} has an app entry which is missing its command",
                        name, &config.platform
                    );
                    valid = false;
                }
            }

            if let Some(svc) = config.service {
                if svc.name.is_empty() {
                    println!(
                        "- {}#{} has a service entry which is missing its domain",
                        name, &config.platform
                    );
                    valid = false;
                }

                if svc.pattern.is_empty() {
                    println!(
                        "- {}#{} has a service entry which is missing its pattern",
                        name, &config.platform
                    );
                    valid = false;
                } else if !valid_service_pattern(&svc.pattern) {
                    println!("- {}#{} has a service entry with an invalid pattern, it should match the regex: /^\\*(\\/\\*)*$/", name, &config.platform);
                    valid = false;
                }

                if svc.website.is_empty() {
                    println!(
                        "- {}#{} has a service entry which is missing its website template",
                        name, &config.platform
                    );
                    valid = false;
                }

                if svc.git_url.is_empty() {
                    println!(
                        "- {}#{} has a service entry which is missing its Git URL template",
                        name, &config.platform
                    );
                    valid = false;
                }

                if valid {
                    let test_service: Service = svc.into();

                    if let Err(err) = test_service.get_website(&test_repo) {
                        println!(
                            "- {}#{} could not render the website URL for a repository: {}",
                            name,
                            &config.platform,
                            err.message()
                        );
                        valid = false;
                    }

                    if let Err(err) = test_service.get_git_url(&test_repo) {
                        println!(
                            "- {}#{} could not render the Git URL for a repository: {}",
                            name,
                            &config.platform,
                            err.message()
                        );
                        valid = false;
                    }
                }
            }
        }

        Ok(valid)
    }

    fn valid_service_pattern(pattern: &str) -> bool {
        let mut expecting_slash = false;
        for c in pattern.chars() {
            if expecting_slash && c != '/' {
                return false;
            }

            if !expecting_slash && c != '*' {
                return false;
            }

            expecting_slash = !expecting_slash;
        }

        true
    }
}
