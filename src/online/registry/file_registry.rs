use super::*;
use crate::fs::to_native_path;
use std::{path::PathBuf, fs::read_to_string, fs::read_dir};

pub struct FileRegistry {
    path: PathBuf
}

impl FileRegistry {
    fn new(path: PathBuf) -> Self {
        Self {
            path
        }
    }
}

impl From<&Config> for FileRegistry {
    fn from(config: &Config) -> Self {
        Self::new(config.get_dev_directory().join("registry"))
    }
}

#[async_trait::async_trait]
impl<'a> Registry<'a> for FileRegistry {
    async fn get_entries(&self) -> Result<Vec<String>, Error> {
        let mut entries = Vec::new();

        for entry in read_dir(self.path.clone())? {
            let type_entry = entry?;

            if !type_entry.path().is_dir() {
                continue;
            }

            for entry in read_dir(type_entry.path())? {
                let file_entry = entry?;

                if file_entry.file_name().to_str().map(|s| s.ends_with(".yaml")).unwrap_or_default() {
                    if let Some(file_name) = PathBuf::from(file_entry.file_name()).file_stem() {
                        entries.push(format!("{}/{}", PathBuf::from(type_entry.file_name()).display(), PathBuf::from(file_name).display()));
                    }
                }
            }
        }

        Ok(entries)
    }

    async fn get_entry(&self, id: &str) -> Result<Entry, Error> {
        let contents = read_to_string(self.path.join(to_native_path(format!("{}.yaml", id))))?;

        Ok(serde_yaml::from_str(&contents)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::get_repo_root;

    #[tokio::test]
    async fn get_entries() {
        let registry = FileRegistry::new(get_repo_root().join("registry"));

        let entries = registry.get_entries().await.unwrap();
        assert_ne!(entries.len(), 0);
        assert!(entries.iter().any(|i| i == "apps/bash"));
    }
    
    #[tokio::test]
    async fn get_entry() {
        let registry = FileRegistry::new(get_repo_root().join("registry"));

        let entry = registry.get_entry("apps/bash").await.unwrap();
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

        let mut valid = true;
        for entry in registry.get_entries().await.unwrap() {
            println!("Validating {}", entry);
            match validate_entry(&registry, &entry).await {
                Ok(v) => {
                    valid = valid && v;
                },
                Err(e) => {
                    println!("{}", e.message());
                    valid = false;
                }
            }
        }

        assert!(valid, "all registry entries should be valid");
    }

    async fn validate_entry(registry: &FileRegistry, name: &str) -> Result<bool, Error> {
        let entry = registry.get_entry(name).await?;
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

        let test_repo = Repo::new("example.com/test/repo", PathBuf::from("/dev/example.com/test/repo"));
        
        for config in entry.configs {
            if config.platform.is_empty() {
                println!("- {} has a config which is missing the platform field", name);
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
                    println!("- {}#{} has an app entry which is missing its name", name, &config.platform);
                    valid = false;
                }

                if app.command.is_empty() {
                    println!("- {}#{} has an app entry which is missing its command", name, &config.platform);
                    valid = false;
                }
            }

            if let Some(svc) = config.service {
                if svc.domain.is_empty() {
                    println!("- {}#{} has a service entry which is missing its domain", name, &config.platform);
                    valid = false;
                }

                if svc.pattern.is_empty() {
                    println!("- {}#{} has a service entry which is missing its pattern", name, &config.platform);
                    valid = false;
                } else if !valid_service_pattern(&svc.pattern) {
                    println!("- {}#{} has a service entry with an invalid pattern, it should match the regex: /^\\*(\\/\\*)*$/", name, &config.platform);
                    valid = false;
                }

                if svc.website.is_empty() {
                    println!("- {}#{} has a service entry which is missing its website template", name, &config.platform);
                    valid = false;
                }

                if svc.http_url.is_empty() {
                    println!("- {}#{} has a service entry which is missing its Git+HTTP template", name, &config.platform);
                    valid = false;
                }

                if svc.git_url.is_empty() {
                    println!("- {}#{} has a service entry which is missing its Git+SSH URL template", name, &config.platform);
                    valid = false;
                }

                if valid {
                    let test_service: Service = svc.into();

                    if let Err(err) = test_service.get_website(&test_repo) {
                        println!("- {}#{} could not render the website URL for a repository: {}", name, &config.platform, err.message());
                        valid = false;
                    }

                    if let Err(err) = test_service.get_git_url(&test_repo) {
                        println!("- {}#{} could not render the Git+SSH URL for a repository: {}", name, &config.platform, err.message());
                        valid = false;
                    }

                    if let Err(err) = test_service.get_http_url(&test_repo) {
                        println!("- {}#{} could not render the Git+HTTP URL for a repository: {}", name, &config.platform, err.message());
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