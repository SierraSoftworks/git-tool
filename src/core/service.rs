use serde::{Serialize, Deserialize};
use super::{Error, Repo, templates};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Service {
    domain: String,
    website: String,
    #[serde(rename = "httpUrl")]
    http_url: String,
    #[serde(rename = "gitUrl")]
    git_url: String,
    pattern: String,
}

impl Service {
    pub fn builder() -> ServiceBuilder {
        ServiceBuilder::default()
    }

    pub fn get_domain(&self) -> String {
        self.domain.clone()
    }

    pub fn get_pattern(&self) -> String {
        self.pattern.clone()
    }

    pub fn get_website(&self, r: &Repo) -> Result<String, Error> {
        templates::render(self.website.clone().as_str(), r.into())
    }

    pub fn get_git_url(&self, r: &Repo) -> Result<String, Error> {
        templates::render(self.git_url.clone().as_str(), r.into())
    }

    pub fn get_http_url(&self, r: &Repo) -> Result<String, Error> {
        templates::render(self.http_url.clone().as_str(), r.into())
    }
}

pub struct ServiceBuilder{
    domain: String,
    website: String,
    http_url: String,
    git_url: String,
    pattern: String,
}

impl Default for ServiceBuilder {
    fn default() -> Self {
        Self {
            domain: Default::default(),
            git_url: Default::default(),
            http_url: Default::default(),
            pattern: Default::default(),
            website: Default::default(),
        }
    }
}

impl ServiceBuilder {
    pub fn with_domain(&mut self, domain: &str) -> &mut ServiceBuilder {
        self.domain = domain.to_string();

        self
    }

    pub fn with_website(&mut self, website: &str) -> &mut ServiceBuilder {
        self.website = website.to_string();

        self
    }

    pub fn with_http_url(&mut self, http_url: &str) -> &mut ServiceBuilder {
        self.http_url = http_url.to_string();

        self
    }

    pub fn with_git_url(&mut self, git_url: &str) -> &mut ServiceBuilder {
        self.git_url = git_url.to_string();

        self
    }

    pub fn with_pattern(&mut self, pattern: &str) -> &mut ServiceBuilder {
        self.pattern = pattern.to_string();

        self
    }
}

impl std::convert::From<&mut ServiceBuilder> for Service {
    fn from(builder: &mut ServiceBuilder) -> Self {
        if builder.domain.is_empty() {
            panic!("cannot construct a service with an empty domain")
        }

        if builder.website.is_empty() {
            panic!("cannot construct a service with an empty website template")
        }

        if builder.git_url.is_empty() {
            panic!("cannot construct a service with an empty Git clone URL template")
        }

        if builder.http_url.is_empty() {
            panic!("cannot construct a service with an empty HTTP clone URL template")
        }

        if builder.pattern.is_empty() {
            panic!("cannot construct a service with an empty directory pattern")
        }

        Self {
            domain: builder.domain.clone(),
            website: builder.website.clone(),
            git_url: builder.git_url.clone(),
            http_url: builder.http_url.clone(),
            pattern: builder.pattern.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Service, Repo};
    use std::{path::PathBuf};

    #[test]
    fn service_builder() {
        let svc: Service = Service::builder()
            .with_domain("github.com")
            .with_pattern("*/*")
            .with_website("https://github.com/{{ .Repo.Namespace }}/{{ .Repo.Name }}")
            .with_git_url("git@github.com/{{ .Repo.Namespace }}/{{ .Repo.Name }}.git")
            .with_http_url("https://github.com/{{ .Repo.Namespace }}/{{ .Repo.Name }}.git")
            .into();

        assert_eq!(svc.get_domain(), "github.com");
        assert_eq!(svc.get_pattern(), "*/*");

        let repo = Repo::new("github.com/sierrasoftworks/git-tool", PathBuf::from("/test"));

        assert_eq!(svc.get_git_url(&repo).unwrap(), "git@github.com/sierrasoftworks/git-tool.git");
        assert_eq!(svc.get_http_url(&repo).unwrap(), "https://github.com/sierrasoftworks/git-tool.git");
        assert_eq!(svc.get_website(&repo).unwrap(), "https://github.com/sierrasoftworks/git-tool");
    }
}