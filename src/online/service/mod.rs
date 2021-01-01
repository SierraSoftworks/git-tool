use crate::core::*;
use async_trait::async_trait;
use std::sync::Arc;

pub mod github;

#[async_trait]
pub trait OnlineService: Send + Sync {
    fn handles(&self, service: &Service) -> bool;
    async fn ensure_created(&self, core: &Core, repo: &Repo) -> Result<(), Error>;
}

pub fn services() -> Vec<Arc<dyn OnlineService>> {
    vec![Arc::new(github::GitHubService::default())]
}
