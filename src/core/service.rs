use super::{templates, Error, Repo};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Service {
    pub name: String,
    pub website: String,
    #[serde(rename = "gitUrl")]
    pub git_url: String,
    pub pattern: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api: Option<ServiceAPI>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServiceAPI {
    pub kind: String,
    pub url: String,
}

impl Service {
    pub fn get_website(&self, r: &Repo) -> Result<String, Error> {
        templates::render(self.website.clone().as_str(), r.into())
    }

    pub fn get_git_url(&self, r: &Repo) -> Result<String, Error> {
        templates::render(self.git_url.clone().as_str(), r.into())
    }
}

impl std::fmt::Display for Service {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.name)
    }
}

#[cfg(test)]
mod tests {
    use crate::core::ServiceAPI;

    use super::{Repo, Service};
    use std::path::PathBuf;

    #[test]
    fn service_builder() {
        let svc = Service {
            name: "gh".into(),
            website: "https://github.com/{{ .Repo.FullName }}".into(),
            git_url: "git@github.com/{{ .Repo.FullName }}.git".into(),
            pattern: "*/*".into(),
            api: Some(ServiceAPI {
                kind: "GitHub/v3".into(),
                url: "https://api.github.com".into(),
            }),
        };

        assert_eq!(&svc.name, "gh");
        assert_eq!(&svc.pattern, "*/*");

        let repo = Repo::new("gh:sierrasoftworks/git-tool", PathBuf::from("/test"));

        assert_eq!(
            &svc.get_git_url(&repo).unwrap(),
            "git@github.com/sierrasoftworks/git-tool.git"
        );
        assert_eq!(
            &svc.get_website(&repo).unwrap(),
            "https://github.com/sierrasoftworks/git-tool"
        );

        let api = svc.api.unwrap();
        assert_eq!(&api.kind, "GitHub/v3");
        assert_eq!(&api.url, "https://api.github.com");
    }

    #[test]
    fn service_uri_encoding() {
        let svc = Service {
            name: "ado".into(),
            website: "https://dev.azure.com/{{ .Repo.Namespace }}/_git/{{ .Repo.Name | urlquery }}"
                .into(),
            git_url: "git@ssh.dev.azure.com:v3/{{ .Repo.FullName | urlquery }}".into(),
            pattern: "*/*/*".into(),
            api: None,
        };

        let repo = Repo::new(
            "ado:sierrasoftworks/example/git tool",
            PathBuf::from("/test"),
        );

        assert_eq!(
            svc.get_git_url(&repo).unwrap(),
            "git@ssh.dev.azure.com:v3/sierrasoftworks/example/git%20tool"
        );
    }
}
