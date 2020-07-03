use crate::core::*;
use async_trait::async_trait;
use std::sync::Arc;

mod github;



#[async_trait]
pub trait OnlineService<
    K: KeyChain = DefaultKeyChain,
    L: Launcher = DefaultLauncher,
    R: Resolver = DefaultResolver,
    O: Output = DefaultOutput,
>: Send + Sync
{
    fn handles(&self, service: &Service) -> bool;
    async fn ensure_created(&self, core: &Core<K, L, R, O>, repo: &Repo) -> Result<(), Error>;
}

pub fn services<K: KeyChain, L: Launcher, R: Resolver, O: Output>(
) -> Vec<Arc<dyn OnlineService<K, L, R, O>>> {
    vec![Arc::new(github::GitHubService::default())]
}
