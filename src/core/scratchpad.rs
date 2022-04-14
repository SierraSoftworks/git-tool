use super::{Config, Target};
use gtmpl::Value;
use std::path;

#[derive(Debug, Clone)]
pub struct Scratchpad {
    full_name: String,
    path: path::PathBuf,
}

impl Target for Scratchpad {
    fn get_name(&self) -> String {
        self.full_name.as_str().to_string()
    }

    fn get_path(&self) -> path::PathBuf {
        path::PathBuf::from(self.path.as_path())
    }

    fn exists(&self) -> bool {
        self.path.is_dir()
    }

    fn template_context(&self, _config: &Config) -> Value {
        self.into()
    }
}

impl Scratchpad {
    pub fn new(full_name: &str, path: path::PathBuf) -> Self {
        Self {
            full_name: full_name.to_string(),
            path,
        }
    }
}

impl std::fmt::Display for Scratchpad {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "scratch:{}", &self.full_name)
    }
}
