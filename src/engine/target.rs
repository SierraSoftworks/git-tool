use super::Config;
use gtmpl::Value;
use human_errors::ResultExt;
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
    pub fn new(keep: bool) -> Result<Self, human_errors::Error> {
        let mut builder = tempfile::Builder::new();
        if keep {
            builder.disable_cleanup(true);
        }

        Ok(Self {
            dir: builder.tempdir().wrap_user_err(
                "Failed to create temporary directory.", &[
                    "Ensure that your system has sufficient permissions and disk space to create temporary files.",
                ])?,
        })
    }

    pub fn close(self) -> Result<(), human_errors::Error> {
        let temp_path = self.dir.path().to_owned();

        self.dir.close().wrap_user_err(
            format!("Unable to remove the temporary directory at '{}', it is likely still in use by another application.", temp_path.display()),
            &["Make sure that you close any open files and then try deleting the directory manually."],
        )
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
