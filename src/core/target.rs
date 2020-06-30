use gtmpl::Value;
use super::Config;

pub trait Target {
    fn get_name(&self) -> String;
    fn get_path(&self) -> std::path::PathBuf;
    fn exists(&self) -> bool;
    fn template_context(&self, config: &Config) -> Value;
}