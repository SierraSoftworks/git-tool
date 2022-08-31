use std::fmt::Display;

use super::Config;
use gtmpl::Value;

pub trait Target: Display {
    fn get_name(&self) -> String;
    fn get_path(&self) -> std::path::PathBuf;
    fn exists(&self) -> bool;
    fn template_context(&self, config: &Config) -> Value;
}
