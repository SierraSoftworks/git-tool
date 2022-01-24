pub use super::{templates::repo_context, Config, Target};
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
        let parts: Vec<&str> = full_name.split("/").collect();

        if parts.len() < 3 {
            panic!("A repository's full name must be composed of a $service/$namespace+/$name");
        }

        Self {
            service: parts[0].to_string(),
            namespace: parts[1..parts.len() - 1].join("/"),
            name: parts[parts.len() - 1].to_string(),
            path,
        }
    }

    pub fn get_full_name(&self) -> String {
        String::default() + self.namespace.as_str() + "/" + self.name.as_str()
    }

    pub fn valid(&self) -> bool {
        self.path.join(".git").is_dir()
    }
}
