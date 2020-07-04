use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use std::vec::Vec;
use std::{path, sync::Arc};

use super::app;
use super::features;
use super::service;
use super::super::errors;
use crate::online::registry::EntryConfig;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    #[serde(skip)]
    config_file: Option<path::PathBuf>,

    #[serde(rename = "directory")]
    dev_directory: path::PathBuf,
    #[serde(default, rename = "scratchpads")]
    scratch_directory: Option<path::PathBuf>,

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

        match from.config_file { Some(path) => into.config_file = Some(path.clone()), None => {} }
        if from.dev_directory.components().count() > 0 { into.dev_directory = from.dev_directory.clone(); }
        match from.scratch_directory { Some(path) => into.scratch_directory = Some(path.clone()), None => {} }
        if !from.services.is_empty() { into.services = from.services.clone(); }
        if !from.apps.is_empty() { into.apps = from.apps.clone(); }
        into.features = from.features;
        
        for (k, v) in from.aliases.iter() {
            into.aliases.insert(k.clone(), v.clone());
        }

        into
    }

    pub fn add(&self, template: EntryConfig) -> Self {
        let mut into = self.clone();

        match template.app {
            Some(app) => {
                into.apps.push(Arc::new(app.into()));
            },
            None => {}
        }

        match template.service {
            Some(svc) => {
                into.services.push(Arc::new(svc.into()));
            },
            None => {}
        }

        into
    }

    #[cfg(test)]
    pub fn for_dev_directory(dir: &path::Path) -> Self {
        Self {
            config_file: None,
            dev_directory: dir.to_path_buf(),
            scratch_directory: None,
            features: features::Features::builder()
                .with_use_http_transport(true)
                .build(),
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

    pub fn from_file(path: &path::Path) -> Result<Self, errors::Error> {
        let f = std::fs::File::open(path)?;

        let mut cfg = Config::from_reader(f)?;
        cfg.config_file = Some(path.to_path_buf());

        Ok(cfg)
    }

    pub fn from_reader<R>(rdr: R) -> Result<Self, errors::Error>
        where R : std::io::Read {
        serde_yaml::from_reader(rdr).map(|x| Config::default().extend(x)).map_err(|e| errors::user_with_internal(
            "We couldn't parse your configuration file.", 
            "Please make sure that the YAML in your configuration file is correctly formatted.", 
            e))
    }
    
    pub fn to_string(&self) -> Result<String, errors::Error>
    {
        serde_yaml::to_string(self).map_err(|e| errors::system_with_internal(
            "We couldn't serialize your configuration to YAML.",
            "Please report this issue on GitHub so that we can try and resolve it.",
            e))
    }

    pub fn get_config_file(&self) -> Option<path::PathBuf> {
        self.config_file.clone()
    }

    pub fn get_dev_directory(&self) -> &path::Path {
        &self.dev_directory
    }

    pub fn get_scratch_directory(&self) -> path::PathBuf {
        match self.scratch_directory.clone() {
            Some(dir) => dir,
            None => self.get_dev_directory().join("scratch")
        }
    }

    pub fn get_apps(&self) -> core::slice::Iter<Arc<app::App>> {
        self.apps.iter()
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

    pub fn get_services(&self) -> core::slice::Iter<Arc<service::Service>> {
        self.services.iter()
    }

    pub fn get_default_service(&self) -> Option<&service::Service> {
        self.services.first().map(|f| f.as_ref())
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

    pub fn get_features(&self) -> &features::Features {
        &self.features
    }
}

impl Default for Config {
    fn default() -> Self {
        let dev_dir = path::PathBuf::from(std::env::var("DEV_DIRECTORY").unwrap_or_default());

        Self {
            config_file: None,
            dev_directory: dev_dir,
            scratch_directory: None,
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
    use crate::online::registry::{EntryApp, EntryConfig, EntryService};

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

    #[test]
    fn add_template() {
        let cfg = Config::default();
        let new_cfg = cfg.add(EntryConfig {
            platform: "linux".to_string(),
            app: Some(EntryApp {
                name: "test-app".to_string(),
                command: "/bin/true".to_string(),
                args: vec![],
                environment: vec![]
            }),
            service: Some(EntryService {
                domain: "example.com".to_string(),
                pattern: "*/*".to_string(),
                website: "https://{{ .Service.Domain }}/{{ .Repo.FullName }}".to_string(),
                git_url: "git@{{ .Service.Domain }}:{{ .Repo.FullName }}.git".to_string(),
                http_url: "https://{{ .Service.Domain }}/{{ .Repo.FullName }}.git".to_string(),
            })
        });

        assert!(new_cfg.get_app("test-app").is_some(), "the test-app should have been added");
        assert!(new_cfg.get_service("example.com").is_some(), "the example service should have been registered");
    }
}