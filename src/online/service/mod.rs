use crate::engine::*;
use crate::errors::HumanErrorResultExt;
use async_trait::async_trait;
use human_errors::ResultExt;
use reqwest::{Method, Request, StatusCode, Url};
use std::sync::Arc;
use std::time::Duration;
use tracing_batteries::prelude::*;

pub mod bitbucket;
pub mod gitea;
pub mod github;
pub mod gitlab;

#[async_trait]
pub trait OnlineService: Send + Sync {
    fn handles(&self, service: &Service) -> bool;
    fn auth_instructions(&self) -> String;
    async fn test(&self, core: &Core, service: &Service) -> Result<(), human_errors::Error>;
    async fn is_created(
        &self,
        core: &Core,
        service: &Service,
        repo: &Repo,
    ) -> Result<bool, human_errors::Error>;
    async fn ensure_created(
        &self,
        core: &Core,
        service: &Service,
        repo: &Repo,
    ) -> Result<(), human_errors::Error>;

    async fn move_repo(
        &self,
        core: &Core,
        service: &Service,
        source: &Repo,
        destination: &Repo,
    ) -> Result<(), human_errors::Error>;

    async fn fork_repo(
        &self,
        core: &Core,
        service: &Service,
        source: &Repo,
        destination: &Repo,
        default_branch_only: bool,
    ) -> Result<(), human_errors::Error>;
}

#[allow(dead_code)]
pub fn services() -> Vec<Arc<dyn OnlineService>> {
    vec![
        Arc::new(github::GitHubService::default()),
        Arc::new(gitlab::GitLabService::default()),
        Arc::new(gitea::GiteaService::default()),
        Arc::new(bitbucket::BitBucketService::default()),
    ]
}

/// Sends an HTTP request to a remote Git hosting service, transparently retrying
/// requests which fail with a transient error (network failures, rate limiting,
/// or server errors) up to a small number of times.
///
/// The caller is responsible for providing all request headers (including the
/// `Accept` and `Authorization` headers) and for interpreting the response body
/// based on the returned [`StatusCode`]. The `User-Agent` header is added
/// automatically.
#[allow(dead_code)]
pub(crate) async fn request_with_retry<B: Into<reqwest::Body> + Clone>(
    core: &Core,
    method: Method,
    uri: &str,
    headers: &[(&'static str, String)],
    body: B,
) -> Result<(StatusCode, Vec<u8>), human_errors::Error> {
    let url: Url = uri.parse().wrap_system_err(
        format!("Unable to parse the API URL '{uri}'."),
        &["Please report this error to us by opening an issue on GitHub."],
    )?;

    let mut remaining_attempts = 3;
    let retryable = [
        StatusCode::TOO_MANY_REQUESTS,
        StatusCode::INTERNAL_SERVER_ERROR,
        StatusCode::BAD_GATEWAY,
        StatusCode::SERVICE_UNAVAILABLE,
    ];

    loop {
        remaining_attempts -= 1;

        let mut req = Request::new(method.clone(), url.clone());
        *req.body_mut() = Some(body.clone().into());

        let req_headers = req.headers_mut();
        req_headers.append(
            "User-Agent",
            version!("Git-Tool/v").parse().to_human_error()?,
        );

        for (name, value) in headers {
            req_headers.append(*name, value.parse().to_human_error()?);
        }

        match core.http_client().request(req).await {
            Ok(resp) if remaining_attempts > 0 && retryable.contains(&resp.status()) => {
                warn!(
                    "API request failed with status code {}. Retrying...",
                    resp.status()
                );

                tokio::time::sleep(Duration::from_secs(1)).await;
                continue;
            }
            Ok(resp) => {
                let status = resp.status();
                let bytes = resp.bytes().await.wrap_system_err(
                    format!(
                        "We received an HTTP {} {} status code from the server and couldn't read the response body.",
                        status.as_u16(),
                        status.canonical_reason().unwrap_or("Unknown")
                    ),
                    &["The service might be having reliability difficulties at the moment, please try again later."],
                )?;

                return Ok((status, bytes.to_vec()));
            }
            Err(error) if remaining_attempts > 0 => {
                warn!("API request failed with error {}. Retrying...", error);

                tokio::time::sleep(Duration::from_secs(1)).await;
                continue;
            }
            Err(error) => {
                return Err(error);
            }
        }
    }
}
