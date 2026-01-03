use crate::engine::*;
use serde::{Deserialize, Serialize};
use std::env::consts::OS;

mod file_registry;
mod github_registry;

#[allow(unused_imports)]
pub use file_registry::FileRegistry;
pub use github_registry::GitHubRegistry;

#[async_trait::async_trait]
pub trait Registry: Send + Sync {
    async fn get_entries(&self, core: &Core) -> Result<Vec<String>, human_errors::Error>;
    async fn get_entry(&self, core: &Core, id: &str) -> Result<Entry, human_errors::Error>;
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Entry {
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub configs: Vec<EntryConfig>,
}

impl std::fmt::Display for Entry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.name)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct EntryConfig {
    pub platform: String,
    #[serde(default)]
    pub app: Option<EntryApp>,
    #[serde(default)]
    pub service: Option<EntryService>,
}

impl EntryConfig {
    pub fn is_compatible(&self) -> bool {
        self.platform == "any" || self.platform == translate_os_name(OS)
    }

    pub fn with_name(self, name: &str) -> EntryConfig {
        EntryConfig {
            platform: self.platform.clone(),
            app: self.app.map(|a| a.with_name(name)),
            service: self.service.map(|s| s.with_name(name)),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct EntryApp {
    pub name: String,
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub environment: Vec<String>,
}

impl EntryApp {
    pub fn with_name(self, name: &str) -> Self {
        EntryApp {
            name: name.to_string(),
            command: self.command.clone(),
            args: self.args,
            environment: self.environment,
        }
    }
}

#[allow(clippy::from_over_into)]
impl Into<App> for EntryApp {
    fn into(self) -> App {
        App::builder()
            .with_name(&self.name)
            .with_command(&self.command)
            .with_args(self.args.iter().map(|s| s.as_str()).collect())
            .with_environment(self.environment.iter().map(|s| s.as_str()).collect())
            .into()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct EntryService {
    pub name: String,
    pub website: String,
    #[serde(rename = "gitUrl")]
    pub git_url: String,
    pub pattern: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api: Option<ServiceAPI>,
}

impl EntryService {
    pub fn with_name(self, name: &str) -> Self {
        EntryService {
            name: name.to_string(),
            website: self.website.clone(),
            git_url: self.git_url.clone(),
            pattern: self.pattern.clone(),
            api: self.api,
        }
    }
}

#[allow(clippy::from_over_into)]
impl Into<Service> for EntryService {
    fn into(self) -> Service {
        Service {
            name: self.name,
            website: self.website,
            git_url: self.git_url,
            pattern: self.pattern,
            api: self.api,
        }
    }
}

fn translate_os_name(name: &str) -> &str {
    match name {
        "macos" => "darwin",
        _ => name,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_compatible() {
        assert!(
            EntryConfig {
                platform: "any".to_string(),
                ..Default::default()
            }
            .is_compatible()
        );

        assert_eq!(
            EntryConfig {
                platform: "windows".to_string(),
                ..Default::default()
            }
            .is_compatible(),
            OS == "windows"
        );

        assert_eq!(
            EntryConfig {
                platform: "linux".to_string(),
                ..Default::default()
            }
            .is_compatible(),
            OS == "linux"
        );

        assert_eq!(
            EntryConfig {
                platform: "darwin".to_string(),
                ..Default::default()
            }
            .is_compatible(),
            OS == "macos"
        );
    }
}
