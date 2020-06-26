use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use std::vec::Vec;
use std::{path::{Path, PathBuf}, sync::Arc};

use super::app;
use super::features;
use super::service;
use super::super::errors;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    #[serde(rename = "directory")]
    dev_directory: String,
    #[serde(default, rename = "scratchpads")]
    scratch_directory: String,

    #[serde(default)]
    services: Vec<Arc<service::Service>>,
    #[serde(default)]
    apps: Vec<Arc<app::App>>,
    #[serde(default)]
    aliases: HashMap<String, String>,

    #[serde(default)]
    features: features::Features,
}

impl Config {
    pub fn extend(&self, other: Self) -> Self {
        let mut into = self.clone();
        let from = other.clone();

        if !from.dev_directory.is_empty() { into.dev_directory = from.dev_directory.clone(); }
        if !from.scratch_directory.is_empty() { into.scratch_directory = from.scratch_directory.clone(); }
        if !from.services.is_empty() { into.services = from.services.clone(); }
        if !from.apps.is_empty() { into.apps = from.apps.clone(); }
        into.features = from.features;
        
        for (k, v) in from.aliases.iter() {
            into.aliases.insert(k.clone(), v.clone());
        }

        into
    }

    #[cfg(test)]
    pub fn for_dev_directory(dir: &Path) -> Self {
        Self {
            dev_directory: dir.to_str().unwrap_or("").to_string(),
            scratch_directory: dir.join("scratch").to_str().unwrap_or("").to_string(),
            ..Default::default()
        }
    }

    #[cfg(test)]
    pub fn from_str(yaml: &str) -> Result<Self, errors::Error> {
        serde_yaml::from_str(yaml).map(|x| Config::default().extend(x)).map_err(|e| errors::user_with_internal(
            "We couldn't parse your configuration file.", 
            "Please make sure that the YAML in your configuration file is correctly formatted.", 
            e))
    }

    pub fn from_reader<R>(rdr: R) -> Result<Self, errors::Error>
        where R : std::io::Read {
        serde_yaml::from_reader(rdr).map(|x| Config::default().extend(x)).map_err(|e| errors::user_with_internal(
            "We couldn't parse your configuration file.", 
            "Please make sure that the YAML in your configuration file is correctly formatted.", 
            e))
    }
    
    pub fn get_dev_directory(&self) -> PathBuf {
        PathBuf::from(self.dev_directory.as_str())
    }

    pub fn get_scratch_directory(&self) -> PathBuf {
        if self.scratch_directory.is_empty() {
            return self.get_dev_directory().join("scratch")
        }

        PathBuf::from(self.scratch_directory.as_str())
    }

    pub fn get_default_app(&self) -> Option<&app::App> {
        self.apps.first().map(|f| f.as_ref())
    }

    pub fn get_app(&self, name: &str) -> Option<&app::App> {
        for app in self.apps.iter() {
            if app.get_name() == name {
                return Some(app.as_ref())
            }
        }

        None
    }

    pub fn get_service(&self, domain: &str) -> Option<&service::Service> {
        for svc in self.services.iter(){
            if svc.get_domain() == domain {
                return Some(svc.as_ref())
            }
        }

        None
    }

    pub fn get_alias(&self, name: &str) -> Option<String> {
        self.aliases.get(name).map(|r| r.clone())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            dev_directory: std::env::var("DEV_DIRECTORY").unwrap_or_default(),
            scratch_directory: Default::default(),
            apps: vec![
                Arc::new(app::App::builder().with_name("shell").with_command("bash").into()),
            ],
            services: vec![
                Arc::new(service::Service::builder()
                    .with_domain("github.com")
                    .with_pattern("*/*")
                    .with_website("https://{{ .Service.Domain }}/{{ .Repo.FullName }}")
                    .with_git_url("git@{{ .Service.Domain }}:{{ .Repo.FullName }}.git")
                    .with_http_url("https://{{ .Service.Domain }}/{{ .Repo.FullName }}.git")
                    .into()),
                Arc::new(service::Service::builder()
                    .with_domain("gitlab.com")
                    .with_pattern("*/*")
                    .with_website("https://{{ .Service.Domain }}/{{ .Repo.FullName }}")
                    .with_git_url("git@{{ .Service.Domain }}:{{ .Repo.FullName }}.git")
                    .with_http_url("https://{{ .Service.Domain }}/{{ .Repo.FullName }}.git")
                    .into()),
                Arc::new(service::Service::builder()
                    .with_domain("bitbucket.org")
                    .with_pattern("*/*")
                    .with_website("https://{{ .Service.Domain }}/{{ .Repo.FullName }}")
                    .with_git_url("git@{{ .Service.Domain }}:{{ .Repo.FullName }}.git")
                    .with_http_url("https://{{ .Service.Domain }}/{{ .Repo.FullName }}.git")
                    .into()),
                Arc::new(service::Service::builder()
                    .with_domain("dev.azure.com")
                    .with_pattern("*/*/*")
                    .with_website("https://{{ .Service.Domain }}/{{ .Repo.Namespace }}/_git/{{ .Repo.Name }}")
                    .with_git_url("git@ssh.{{ .Service.Domain }}:v3/{{ .Repo.FullName }}.git")
                    .with_http_url("https://{{ .Service.Domain }}/{{ .Repo.Namespace }}/_git/{{ .Repo.Name }}.git")
                    .into()),
            ],
            aliases: HashMap::new(),
            features: Default::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Config;
    use std::path::PathBuf;

    #[test]
    fn load_from_string_basic() {
        match Config::from_str("directory: /test/dev") {
            Ok(cfg) => {
                assert_eq!(cfg.get_dev_directory(),  PathBuf::from("/test/dev"));
                assert_eq!(cfg.get_scratch_directory(), PathBuf::from("/test/dev/scratch"));

                match cfg.get_app("shell") {
                    Some(app) => {
                        assert_eq!(app.get_name(), "shell");
                        assert_eq!(app.get_command(), "bash");
                    },
                    _ => panic!("expected that the shell app would be registered")
                }
            },
            Err(e) => {
                panic!(e.message())
            }
        }
    }
    
    #[test]
    fn load_from_string_with_scratchdir() {
        match Config::from_str("directory: /test/dev\nscratchpads: /test/scratch") {
            Ok(cfg) => {
                assert_eq!(cfg.get_dev_directory(), PathBuf::from("/test/dev"));
                assert_eq!(cfg.get_scratch_directory(), PathBuf::from("/test/scratch"));

                match cfg.get_service("github.com") {
                    Some(_) => {},
                    None => panic!("The default services should be present.")
                }
                
                match cfg.get_app("shell") {
                    Some(_) => {},
                    None => panic!("The default apps should be present.")
                }
            },
            Err(e) => {
                panic!(e.message())
            }
        }
    }
    
    #[test]
    fn load_from_string_with_new_apps() {
        match Config::from_str("
directory: /test/dev

apps:
    - name: test-app
      command: test
") {
            Ok(cfg) => {
                assert_eq!(cfg.get_dev_directory(), PathBuf::from("/test/dev"));

                match cfg.get_service("github.com") {
                    Some(_) => {},
                    None => panic!("The default services should be present.")
                }
                
                match cfg.get_app("test-app") {
                    Some(_) => {},
                    None => panic!("The new apps should be present.")
                }
                
                match cfg.get_app("shell") {
                    Some(_) => panic!("The default apps should not be present."),
                    None => {},
                }
            },
            Err(e) => {
                panic!(e.message())
            }
        }
    }
}