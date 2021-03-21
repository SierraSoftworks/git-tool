use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::Path};
use std::{env::consts::OS, vec::Vec};
use std::{path, sync::Arc};

use super::super::errors;
use super::app;
use super::features;
use super::service;
use crate::online::registry::EntryConfig;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    #[serde(skip)]
    config_file: Option<path::PathBuf>,

    #[serde(rename = "$schema")]
    schema: Option<String>,

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
    pub fn with_dev_directory(&self, dev_dir: &Path) -> Self {
        let mut into = self.clone();
        into.dev_directory = dev_dir.to_owned();
        into
    }

    pub fn with_feature_flag(&self, flag: &str, enabled: bool) -> Self {
        let mut into = self.clone();
        into.features = self.features.to_builder().with(flag, enabled).build();
        into
    }

    pub fn extend(&self, other: Self) -> Self {
        let mut into = self.clone();
        let from = other.clone();

        match from.config_file {
            Some(path) => into.config_file = Some(path.clone()),
            None => {}
        }
        if from.dev_directory.components().count() > 0 {
            into.dev_directory = from.dev_directory.clone();
        }
        match from.scratch_directory {
            Some(path) => into.scratch_directory = Some(path.clone()),
            None => {}
        }
        if !from.services.is_empty() {
            into.services = from.services.clone();
        }
        if !from.apps.is_empty() {
            into.apps = from.apps.clone();
        }

        into.features = features::Features::builder()
            .with_features(&into.features)
            .with_features(&from.features)
            .build();

        for (k, v) in from.aliases.iter() {
            into.aliases.insert(k.clone(), v.clone());
        }

        into
    }

    pub fn apply_template(
        &self,
        template: EntryConfig,
        replace_existing: bool,
    ) -> Result<Self, errors::Error> {
        let mut into = self.clone();

        match template.app {
            Some(app) => {
                if let Some(existing_position) =
                    into.apps.iter().position(|a| a.get_name() == app.name)
                {
                    if !replace_existing {
                        return Err(errors::user(
                            &format!("The application {} already exists in your configuration file. Adding a duplicate entry will have no effect.", &app.name),
                            &format!("If you would like to replace the existing entry for {app}, use `gt config add apps/{app} --force`.", app=&app.name)));
                    } else {
                        into.apps[existing_position] = Arc::new(app.into());
                    }
                } else {
                    into.apps.push(Arc::new(app.into()));
                }
            }
            None => {}
        }

        match template.service {
            Some(svc) => {
                if let Some(existing_position) = into
                    .services
                    .iter()
                    .position(|s| s.get_domain() == svc.domain)
                {
                    if !replace_existing {
                        return Err(errors::user(
                            &format!("The service {} already exists in your configuration file. Adding a duplicate entry will have no effect.", &svc.domain),
                            &format!("If you would like to replace the existing entry for {svc}, use `gt config add services/{svc} --force`.", svc=&svc.domain)));
                    } else {
                        into.services[existing_position] = Arc::new(svc.into());
                    }
                } else {
                    into.services.push(Arc::new(svc.into()));
                }
            }
            None => {}
        }

        Ok(into)
    }

    #[cfg(test)]
    pub fn for_dev_directory(dir: &path::Path) -> Self {
        Self {
            config_file: None,
            dev_directory: dir.to_path_buf(),
            scratch_directory: None,
            features: features::Features::builder()
                .with_defaults()
                .with(features::HTTP_TRANSPORT, true)
                .build(),
            ..Default::default()
        }
    }

    #[cfg(test)]
    pub fn from_str(yaml: &str) -> Result<Self, errors::Error> {
        serde_yaml::from_str(yaml)
            .map(|x| Config::default().extend(x))
            .map_err(|e| {
                errors::user_with_internal(
                    "We couldn't parse your configuration file due to a YAML parser error.",
                    "Check that the YAML in your configuration file is correctly formatted.",
                    e,
                )
            })
    }

    pub fn from_file(path: &path::Path) -> Result<Self, errors::Error> {
        let f = std::fs::File::open(path).map_err(|err| errors::user_with_internal(
            &format!("We could not open your Git-Tool config file '{}' for reading.", path.display()),
            "Check that your config file exists and is readable by the user running git-tool before trying again.",
            err
        ))?;

        let mut cfg = Config::default().extend(Config::from_reader(f)?);
        cfg.config_file = Some(path.to_path_buf());

        Ok(cfg)
    }

    pub fn from_reader<R>(rdr: R) -> Result<Self, errors::Error>
    where
        R: std::io::Read,
    {
        serde_yaml::from_reader(rdr)
            .map(|x| Config::default().extend(x))
            .map_err(|e| {
                errors::user_with_internal(
                    "We couldn't parse your configuration file due to a YAML parser error.",
                    "Check that the YAML in your configuration file is correctly formatted.",
                    e,
                )
            })
    }

    pub fn to_string(&self) -> Result<String, errors::Error> {
        let config = serde_yaml::to_string(self).map_err(|e| {
            errors::system_with_internal(
                "We couldn't serialize your configuration to YAML due to a YAML serializer error.",
                "Please report this issue on GitHub so that we can try and resolve it.",
                e,
            )
        })?;

        match &self.schema {
            Some(schema) => Ok(format!(
                "# yaml-language-server: $schema={}\n{}",
                schema, config
            )),
            None => Ok(config),
        }
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
            None => self.get_dev_directory().join("scratch"),
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
                return Some(app.as_ref());
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
        for svc in self.services.iter() {
            if svc.get_domain() == domain {
                return Some(svc.as_ref());
            }
        }

        None
    }

    pub fn get_alias(&self, name: &str) -> Option<String> {
        self.aliases.get(name).map(|r| r.clone())
    }

    pub fn get_aliases(&self) -> std::collections::hash_map::Iter<String, String> {
        self.aliases.iter()
    }

    pub fn add_alias(&mut self, name: &str, repo: &str) {
        self.aliases.insert(name.to_string(), repo.to_string());
    }

    pub fn remove_alias(&mut self, name: &str) {
        self.aliases.remove(name);
    }

    pub fn get_features(&self) -> &features::Features {
        &self.features
    }
}

impl Default for Config {
    fn default() -> Self {
        let dev_dir = path::PathBuf::from(std::env::var("DEV_DIRECTORY").unwrap_or_default());

        let default_shell = match OS {
            "linux" => "bash",
            "macos" => "zsh",
            "windows" => "powershell",
            _ => "bash",
        };

        Self {
            schema: Some("https://schemas.sierrasoftworks.com/git-tool/v1/config.schema.json".into()),
            config_file: None,
            dev_directory: dev_dir,
            scratch_directory: None,
            apps: vec![
                Arc::new(app::App::builder().with_name("shell").with_command(default_shell).into()),
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
                    .with_git_url("git@ssh.{{ .Service.Domain }}:v3/{{ .Repo.FullName }}")
                    .with_http_url("https://{{ .Service.Domain }}/{{ .Repo.Namespace }}/_git/{{ .Repo.Name }}")
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
    use crate::{
        online::registry::{EntryApp, EntryConfig, EntryService},
        test::get_repo_root,
    };
    use std::path::PathBuf;

    #[test]
    fn load_from_string_basic() {
        match Config::from_str("directory: /test/dev") {
            Ok(cfg) => {
                assert_eq!(cfg.get_dev_directory(), PathBuf::from("/test/dev"));
                assert_eq!(
                    cfg.get_scratch_directory(),
                    PathBuf::from("/test/dev/scratch")
                );

                match cfg.get_app("shell") {
                    Some(app) => {
                        assert_eq!(app.get_name(), "shell");
                    }
                    _ => panic!("expected that the shell app would be registered"),
                }
            }
            Err(e) => panic!("{}", e.message()),
        }
    }

    #[test]
    fn load_from_string_with_scratchdir() {
        match Config::from_str("directory: /test/dev\nscratchpads: /test/scratch") {
            Ok(cfg) => {
                assert_eq!(cfg.get_dev_directory(), PathBuf::from("/test/dev"));
                assert_eq!(cfg.get_scratch_directory(), PathBuf::from("/test/scratch"));

                match cfg.get_service("github.com") {
                    Some(_) => {}
                    None => panic!("The default services should be present."),
                }

                match cfg.get_app("shell") {
                    Some(_) => {}
                    None => panic!("The default apps should be present."),
                }
            }
            Err(e) => panic!("{}", e.message()),
        }
    }

    #[test]
    fn load_from_string_with_new_apps() {
        match Config::from_str(
            "
directory: /test/dev

apps:
    - name: test-app
      command: test
",
        ) {
            Ok(cfg) => {
                assert_eq!(cfg.get_dev_directory(), PathBuf::from("/test/dev"));

                match cfg.get_service("github.com") {
                    Some(_) => {}
                    None => panic!("The default services should be present."),
                }

                match cfg.get_app("test-app") {
                    Some(_) => {}
                    None => panic!("The new apps should be present."),
                }

                match cfg.get_app("shell") {
                    Some(_) => panic!("The default apps should not be present."),
                    None => {}
                }
            }
            Err(e) => panic!("{}", e.message()),
        }
    }

    #[test]
    fn apply_template() {
        let cfg = Config::default();
        let template = EntryConfig {
            platform: "linux".to_string(),
            app: Some(EntryApp {
                name: "test-app".to_string(),
                command: "/bin/true".to_string(),
                args: vec![],
                environment: vec![],
            }),
            service: Some(EntryService {
                domain: "example.com".to_string(),
                pattern: "*/*".to_string(),
                website: "https://{{ .Service.Domain }}/{{ .Repo.FullName }}".to_string(),
                git_url: "git@{{ .Service.Domain }}:{{ .Repo.FullName }}.git".to_string(),
                http_url: "https://{{ .Service.Domain }}/{{ .Repo.FullName }}.git".to_string(),
            }),
        };

        let new_cfg = cfg.apply_template(template.clone(), false).unwrap();
        assert!(
            new_cfg.get_app("test-app").is_some(),
            "the test-app should have been added"
        );
        assert!(
            new_cfg.get_service("example.com").is_some(),
            "the example service should have been registered"
        );

        assert!(new_cfg.apply_template(template.clone(), false).is_err());
        assert!(new_cfg.apply_template(template.clone(), true).is_ok());
    }

    #[test]
    fn test_load_file() {
        let file_path = get_repo_root()
            .join("test")
            .join("data")
            .join("config.valid.yml");
        let cfg = Config::from_file(&file_path).unwrap();

        assert!(
            cfg.get_app("make").is_some(),
            "the correct config file should have been loaded"
        );
        assert_eq!(
            cfg.get_alias("gt"),
            Some("github.com/SierraSoftworks/git-tool".to_string()),
            "the aliases should have been loaded"
        );
        assert_eq!(
            cfg.get_config_file(),
            Some(file_path),
            "the file path should have been populated"
        );
    }
}
