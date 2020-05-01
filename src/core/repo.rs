use std::{path};
pub use super::Target;

#[derive(Debug, Clone)]
pub struct Repo {
    domain: String,
    namespace: String,
    name: String,
    path: path::PathBuf,
}

impl Target for Repo {
    fn get_name(&self) -> String {
        self.get_full_name()
    }

    fn get_path(&self) -> path::PathBuf {
        path::PathBuf::from(self.path.as_path())
    }

    fn exists(&self) -> bool {
        self.path.is_dir()
    }
}

impl Repo {
    pub fn new(full_name: &str, path: path::PathBuf) -> Self {
        let parts: Vec<&str> = full_name.split("/").collect();

        if parts.len() < 3 {
            panic!("A repository's full name must be composed of a $domain/$namespace/$name");
        }

        Self {
            domain: parts[0].to_string(),
            namespace: parts[1..parts.len() - 1].join("/"),
            name: parts[parts.len()-1].to_string(),
            path
        }
    }

    pub fn get_domain(&self) -> String {
        self.domain.clone()
    }

    pub fn get_full_name(&self) -> String {
        String::default() + self.namespace.as_str() + "/" + self.name.as_str()
    }

    pub fn get_namespace(&self) -> String {
        self.namespace.clone()
    }

    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    pub fn valid(&self) -> bool {
        self.path.join(".git").is_dir()
    }
}