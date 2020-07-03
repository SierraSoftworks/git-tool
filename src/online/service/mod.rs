use crate::core::*;
use async_trait::async_trait;
use std::sync::Arc;

pub mod github;

#[async_trait]
pub trait OnlineService<C: Core>: Send + Sync {
    fn handles(&self, service: &Service) -> bool;
    async fn ensure_created(&self, core: &C, repo: &Repo) -> Result<(), Error>;
}

pub fn services<C: Core>() -> Vec<Arc<dyn OnlineService<C>>> {
    vec![Arc::new(github::GitHubService::default())]
}
