use std::fmt::Display;

use super::Config;
use gtmpl::Value;

pub trait Target: Display {
    fn get_name(&self) -> String;
    fn get_path(&self) -> std::path::PathBuf;
    fn exists(&self) -> bool;
    fn template_context(&self, config: &Config) -> Value;
}

pub struct TempTarget {
    dir: tempfile::TempDir,
}

impl TempTarget {
    pub fn new() -> Result<Self, crate::errors::Error> {
        Ok(Self {
            dir: tempfile::tempdir()?,
        })
    }

    #[cfg(test)]
    pub fn with_dir(dir: tempfile::TempDir) -> Self {
        Self { dir }
    }
}

impl Target for TempTarget {
    fn get_name(&self) -> String {
        "temp".to_string()
    }

    fn get_path(&self) -> std::path::PathBuf {
        self.dir.path().to_owned()
    }

    fn exists(&self) -> bool {
        true
    }

    fn template_context(&self, _config: &Config) -> Value {
        self.into()
    }
}

impl Display for TempTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "temp")
    }
}
