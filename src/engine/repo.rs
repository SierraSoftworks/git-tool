pub use super::{Config, Target, templates::repo_context};
use gtmpl::Value;
use std::path;

#[derive(Debug, Clone)]
pub struct Repo {
    pub service: String,
    pub namespace: String,
    pub name: String,
    pub path: path::PathBuf,
}

impl Target for Repo {
    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn get_path(&self) -> path::PathBuf {
        path::PathBuf::from(self.path.as_path())
    }

    fn exists(&self) -> bool {
        self.path.is_dir()
    }

    fn template_context(&self, config: &Config) -> Value {
        repo_context(config, self)
    }
}

impl Repo {
    pub fn new(full_name: &str, path: path::PathBuf) -> Self {
        if let Some((svc, relative_name)) = full_name.split_once(':') {
            let parts: Vec<&str> = relative_name.split('/').collect();

            if parts.len() < 2 {
                panic!("A repository's full name must be composed of a $service:$namespace+/$name");
            }

            Self {
                service: svc.to_string(),
                namespace: parts[0..parts.len() - 1].join("/"),
                name: parts[parts.len() - 1].to_string(),
                path,
            }
        } else {
            panic!("A repository's full name must be composed of a $service:$namespace+/$name");
        }
    }

    pub fn get_full_name(&self) -> String {
        String::default() + self.namespace.as_str() + "/" + self.name.as_str()
    }

    pub fn valid(&self) -> bool {
        self.path.join(".git").is_dir()
    }
}

impl std::fmt::Display for Repo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", &self.service, self.get_full_name())
    }
}
