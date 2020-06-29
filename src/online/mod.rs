pub mod gitignore;
pub mod registry;
mod github_registry;
mod file_registry;

use super::errors;
use super::errors::Error;
pub use github_registry::GitHubRegistry;