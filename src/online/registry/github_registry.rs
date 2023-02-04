use super::*;
use crate::errors;
use serde::Deserialize;

pub struct GitHubRegistry;

impl GitHubRegistry {
    async fn get(&self, core: &Core, url: &str) -> Result<reqwest::Response, errors::Error> {
        let uri: reqwest::Url = url.parse().map_err(|e| {
            errors::system_with_internal(
                &format!("Unable to parse GitHub API URL '{url}'."),
                "Please report this error to us by opening a ticket in GitHub.",
                e,
            )
        })?;

        // NOTE: This allows us to consume the GITHUB_TOKEN environment variable in the test
        // environment to bypass rate limiting restrictions.
        // TODO: We should probably support using the users github.com token here to avoid rate limiting
        #[allow(unused_mut)]
        let mut req = reqwest::Request::new(reqwest::Method::GET, uri);

        req.headers_mut().append(
            "User-Agent",
            version!("Git-Tool/").parse().map_err(|e| {
                errors::system_with_internal(
                    &format!(
                        "Unable to parse Git-Tool user agent header {}.",
                        version!("Git-Tool/")
                    ),
                    "Please report this error to us by opening a ticket in GitHub.",
                    e,
                )
            })?,
        );

        #[cfg(test)]
        {
            if let Ok(token) = std::env::var("GITHUB_TOKEN") {
                req.headers_mut().append(
                    "Authorization",
                    format!("token {token}").parse().map_err(|e| {
                        errors::system_with_internal(
                            "Unable to parse GITHUB_TOKEN authorization header.",
                            "Please report this error to us by opening a ticket in GitHub.",
                            e,
                        )
                    })?,
                );
            }
        }

        core.http_client().request(req).await
    }
}

#[async_trait::async_trait]
impl Registry for GitHubRegistry {
    #[tracing::instrument(err, skip(self, core))]
    async fn get_entries(&self, core: &Core) -> Result<Vec<String>, Error> {
        let resp = self.get(core, "https://api.github.com/repos/SierraSoftworks/git-tool/git/trees/main?recursive=true").await?;

        match resp.status() {
            http::StatusCode::OK => {
                let tree: GitHubTree = resp.json().await?;

                let mut entries: Vec<String> = Vec::new();

                let prefix = "registry/";
                let suffix = ".yaml";

                for node in tree.tree {
                    if node.node_type == "blob"
                        && node.path.starts_with(prefix)
                        && node.path.ends_with(suffix)
                    {
                        let len = node.path.len();
                        let name: String = node.path[prefix.len()..(len - suffix.len())].into();
                        debug!("Found entry '{}'", &name);
                        entries.push(name);
                    }
                }

                Ok(entries)
            }
            http::StatusCode::TOO_MANY_REQUESTS | http::StatusCode::FORBIDDEN => {
                let inner_error = errors::reqwest::ResponseError::with_body(resp).await;
                Err(errors::user_with_internal(
                    "GitHub has rate limited requests from your IP address.",
                    "Please wait until GitHub removes this rate limit before trying again.",
                    inner_error,
                ))
            }
            status => {
                let inner_error = errors::reqwest::ResponseError::with_body(resp).await;
                Err(errors::system_with_internal(
                    &format!("Received an HTTP {status} response from GitHub when attempting to list items in the Git-Tool registry."),
                    "Please read the error message below and decide if there is something you can do to fix the problem, or report it to us on GitHub.",
                    inner_error))
            }
        }
    }

    #[tracing::instrument(err, skip(self, core))]
    async fn get_entry(&self, core: &Core, id: &str) -> Result<Entry, Error> {
        let resp = self
            .get(
                core,
                &format!(
            "https://raw.githubusercontent.com/SierraSoftworks/git-tool/main/registry/{id}.yaml"
        ),
            )
            .await?;

        match resp.status() {
            http::StatusCode::OK => {
                let body = resp.bytes().await?;
                let entity = serde_yaml::from_slice(&body)?;
                debug!("{}", entity);
                Ok(entity)
            },
            http::StatusCode::NOT_FOUND => {
                Err(errors::user(
                    &format!("Could not find {id} in the Git-Tool registry."),
                    "Please make sure that you've selected a configuration entry which exists in the registry. You can check this with `git-tool config list`."))
            },
            http::StatusCode::TOO_MANY_REQUESTS | http::StatusCode::FORBIDDEN => {
                let inner_error = errors::reqwest::ResponseError::with_body(resp).await;
                Err(errors::user_with_internal(
                    "GitHub has rate limited requests from your IP address.",
                    "Please wait until GitHub removes this rate limit before trying again.",
                    inner_error))
            },
            status => {
                let inner_error = errors::reqwest::ResponseError::with_body(resp).await;
                Err(errors::system_with_internal(
                    &format!("Received an HTTP {status} response from GitHub when attempting to fetch /registry/{id}.yaml."),
                    "Please read the error message below and decide if there is something you can do to fix the problem, or report it to us on GitHub.",
                    inner_error))
            }
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
struct GitHubTree {
    pub tree: Vec<GitHubTreeNode>,
    pub truncated: bool,
}

#[derive(Debug, Deserialize, Clone)]
struct GitHubTreeNode {
    #[serde(rename = "type")]
    pub node_type: String,
    pub path: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[cfg_attr(feature = "pure-tests", ignore)]
    async fn get_entries() {
        let core = Core::builder().with_default_config().build();
        let registry = GitHubRegistry;

        let entries = registry.get_entries(&core).await.unwrap();
        assert_ne!(entries.len(), 0);
        assert!(entries.iter().any(|i| i == "apps/bash"));
    }

    #[tokio::test]
    #[cfg_attr(feature = "pure-tests", ignore)]
    async fn get_entry() {
        let core = Core::builder().with_default_config().build();
        let registry = GitHubRegistry;

        let entry = registry.get_entry(&core, "apps/bash").await.unwrap();
        assert_eq!(entry.name, "Bash");
    }
}
