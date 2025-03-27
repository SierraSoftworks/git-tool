use super::Config;
use gtmpl::Value;
use std::fmt::Display;

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
    pub fn new(keep: bool) -> Result<Self, crate::errors::Error> {
        Ok(Self {
            dir: tempfile::Builder::new().keep(keep).tempdir()?,
        })
    }

    pub fn close(self) -> Result<(), crate::errors::Error> {
        let temp_path = self.dir.path().to_owned();

        self.dir.close().map_err(|e| crate::errors::user_with_internal(
          "Unable to remove the temporary directory, it is likely still in use by another application.",
          &format!("Make sure that you close any open files and then delete '{}'", temp_path.display()),
          e))?;
        Ok(())
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
