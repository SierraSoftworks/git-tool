use crate::engine::*;
use async_trait::async_trait;
use std::sync::Arc;

pub mod github;

#[async_trait]
pub trait OnlineService: Send + Sync {
    fn handles(&self, service: &Service) -> bool;
    fn auth_instructions(&self) -> String;
    async fn test(&self, core: &Core, service: &Service) -> Result<(), Error>;
    async fn is_created(&self, core: &Core, service: &Service, repo: &Repo) -> Result<bool, Error>;
    async fn ensure_created(
        &self,
        core: &Core,
        service: &Service,
        repo: &Repo,
    ) -> Result<(), Error>;

    async fn move_repo(
        &self,
        core: &Core,
        service: &Service,
        source: &Repo,
        destination: &Repo,
    ) -> Result<(), Error>;

    async fn fork_repo(
        &self,
        core: &Core,
        service: &Service,
        source: &Repo,
        destination: &Repo,
        default_branch_only: bool,
    ) -> Result<(), Error>;
}

#[allow(dead_code)]
pub fn services() -> Vec<Arc<dyn OnlineService>> {
    vec![Arc::new(github::GitHubService::default())]
}
