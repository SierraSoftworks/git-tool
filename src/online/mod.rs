pub mod gitignore;
pub mod registry;
pub mod service;

use super::errors;
use super::errors::Error;
pub use registry::GitHubRegistry;
pub use service::{OnlineService, services};