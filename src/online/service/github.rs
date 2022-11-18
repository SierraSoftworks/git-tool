use std::time::Duration;

use super::*;
use crate::errors;
use reqwest::{Method, Request, StatusCode, Url};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

#[derive(Default)]
pub struct GitHubService {}

#[async_trait]
impl OnlineService for GitHubService {
    fn handles(&self, service: &Service) -> bool {
        service
            .api
            .as_ref()
            .map(|api| api.kind == "GitHub/v3")
            .unwrap_or(false)
    }

    async fn test(&self, core: &Core, service: &Service) -> Result<(), Error> {
        self.get_user_login(core, service).await?;
        Ok(())
    }

    #[tracing::instrument(err, skip(self, core))]
    async fn is_created(&self, core: &Core, service: &Service, repo: &Repo) -> Result<bool, Error> {
        let uri = format!(
            "{}/repos/{}",
            service.api.as_ref().unwrap().url.as_str(),
            repo.get_full_name()
        );
        let repo_resp: Result<NewRepoResponse, GitHubErrorResponse> = self
            .make_request(
                core,
                service,
                Method::GET,
                &uri,
                Vec::new(),
                vec![StatusCode::OK, StatusCode::MOVED_PERMANENTLY],
            )
            .await?;

        match repo_resp {
            Ok(_) => Ok(true),
            Err(e) if e.http_status_code == StatusCode::NOT_FOUND => Ok(false),
            Err(e) => Err(e.into()),
        }
    }

    #[tracing::instrument(err, skip(self, core))]
    async fn ensure_created(
        &self,
        core: &Core,
        service: &Service,
        repo: &Repo,
    ) -> Result<(), Error> {
        let current_user = self.get_user_login(core, service).await?;

        let uri = if repo.namespace == current_user {
            format!("{}/user/repos", service.api.as_ref().unwrap().url.as_str(),)
        } else {
            format!(
                "{}/orgs/{}/repos",
                service.api.as_ref().unwrap().url.as_str(),
                &repo.namespace
            )
        };

        let new_repo = NewRepo {
            name: repo.get_name(),
            private: core
                .config()
                .get_features()
                .has(features::CREATE_REMOTE_PRIVATE),
        };

        let req_body = serde_json::to_vec(&new_repo)?;
        let new_repo_resp: Result<NewRepoResponse, GitHubErrorResponse> = self
            .make_request(
                core,
                service,
                Method::POST,
                &uri,
                req_body,
                vec![StatusCode::CREATED],
            )
            .await?;

        match new_repo_resp {
            Ok(_) => Ok(()),
            Err(e) if e.http_status_code == StatusCode::UNPROCESSABLE_ENTITY => Ok(()),
            Err(e) => Err(e.into()),
        }
    }
}

impl GitHubService {
    async fn get_user_login(&self, core: &Core, service: &Service) -> Result<String, Error> {
        let user: Result<UserProfile, GitHubErrorResponse> = self
            .make_request(
                core,
                service,
                Method::GET,
                &format!("{}/user", service.api.as_ref().unwrap().url.as_str(),),
                "",
                vec![StatusCode::OK],
            )
            .await?;

        match user {
            Ok(user) => Ok(user.login),
            Err(e) => Err(e.into()),
        }
    }

    async fn make_request<B: Into<reqwest::Body> + Clone, T: DeserializeOwned>(
        &self,
        core: &Core,
        service: &Service,
        method: Method,
        uri: &str,
        body: B,
        acceptable: Vec<StatusCode>,
    ) -> Result<Result<T, GitHubErrorResponse>, Error> {
        let url: Url = uri.parse().map_err(|e| {
            errors::system_with_internal(
                &format!("Unable to parse GitHub API URL '{}'.", uri),
                "Please report this error to us by opening an issue on GitHub.",
                e,
            )
        })?;

        let token = core.keychain().get_token(&service.name)?;

        let mut remaining_attempts = 3;
        let retryable = vec![
            StatusCode::TOO_MANY_REQUESTS,
            StatusCode::INTERNAL_SERVER_ERROR,
            StatusCode::BAD_GATEWAY,
            StatusCode::SERVICE_UNAVAILABLE,
        ];

        loop {
            remaining_attempts -= 1;

            let mut req = Request::new(method.clone(), url.clone());

            *req.body_mut() = Some(body.clone().into());

            let headers = req.headers_mut();

            headers.append("User-Agent", version!("Git-Tool/v").parse()?);
            headers.append("Accept", "application/vnd.github.v3+json".parse()?);
            headers.append("Authorization", format!("token {}", token).parse()?);

            match core.http_client().request(req).await {
                Ok(resp) if acceptable.contains(&resp.status()) => {
                    let result = resp.json().await?;

                    return Ok(Ok(result));
                }
                Ok(resp) if remaining_attempts > 0 && retryable.contains(&resp.status()) => {
                    tracing::warn!(
                        "GitHub API request failed with status code {}. Retrying...",
                        resp.status()
                    );

                    tokio::time::sleep(Duration::from_secs(1)).await;

                    continue;
                }
                Ok(resp) => {
                    let status = resp.status();
                    let mut result: GitHubErrorResponse = resp.json().await?;
                    result.http_status_code = status;

                    return Ok(Err(result));
                }
                Err(error) if remaining_attempts > 0 => {
                    tracing::warn!(
                        "GitHub API request failed with error {}. Retrying...",
                        error
                    );

                    tokio::time::sleep(Duration::from_secs(1)).await;

                    continue;
                }
                Err(error) => {
                    return Err(error);
                }
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct NewRepo {
    pub name: String,
    pub private: bool,
}

#[derive(Debug, Deserialize)]
struct UserProfile {
    pub login: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct NewRepoResponse {
    pub id: u64,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct GitHubErrorResponse {
    #[serde(skip)]
    pub http_status_code: StatusCode,

    pub message: String,
    pub documentation_url: String,
    #[serde(default)]
    pub errors: Vec<GitHubError>,
}

#[allow(clippy::from_over_into)]
impl Into<errors::Error> for GitHubErrorResponse {
    fn into(self) -> errors::Error {
        match self.http_status_code {
            http::StatusCode::UNAUTHORIZED => {
                errors::user(
                    "You have not provided a valid authentication token for github.com.",
                    "Please generate a valid Personal Access Token at https://github.com/settings/tokens (with the `repo` scope) and add it using `git-tool auth github.com`.")
            },
            http::StatusCode::FORBIDDEN => {
                errors::user_with_internal(
                    &format!("You do not have permission to perform this action on GitHub: {}", self.message),
                    "Check your GitHub account permissions for this organization or repository and try again.",
                    errors::detailed_message(&format!("{:?}", self)),
                )
            },
            http::StatusCode::NOT_FOUND => {
                errors::user_with_internal(
                    "We could not create the GitHub repo because the organization or user you specified could not be found.",
                    "Check that you have specified the correct organization or user in the repository name and try again.",
                    errors::detailed_message(&format!("{:?}", self))
                )
            },
            http::StatusCode::TOO_MANY_REQUESTS => {
                errors::user(
                    "GitHub has rate limited requests from your IP address.",
                    "Please wait until GitHub removes this rate limit before trying again.")
            },
            status => {
                errors::system_with_internal(
                    &format!("Received an HTTP {} {} response from GitHub.", status.as_u16(), status.canonical_reason().unwrap_or_default()),
                    "Please read the error message below and decide if there is something you can do to fix the problem, or report it to us on GitHub.",
                    errors::detailed_message(&format!("{:?}", self)))
            }
        }
    }
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct GitHubError {
    pub message: String,

    #[serde(default)]
    pub resource: String,

    #[serde(default)]
    pub code: String,

    #[serde(default)]
    pub field: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::eq;

    #[tokio::test]
    async fn test_happy_path_user_repo() {
        let core = Core::builder()
            .with_default_config()
            .with_mock_keychain(|mock| {
                mock.expect_get_token()
                    .with(eq("gh"))
                    .returning(|_| Ok("test_token".into()));
            })
            .with_mock_http_client(mocks::repo_created("test"))
            .build();

        let repo = Repo::new("gh:test/user-repo", std::path::PathBuf::from("/"));
        let service = GitHubService::default();
        service
            .ensure_created(
                &core,
                &Service {
                    name: "gh".into(),
                    website: "https://github.com/{{ .Repo.FullName }}".into(),
                    git_url: "git@github.com/{{ .Repo.FullName }}.git".into(),
                    pattern: "*/*".into(),
                    api: Some(ServiceAPI {
                        kind: "github".into(),
                        url: "https://api.github.com".into(),
                    }),
                },
                &repo,
            )
            .await
            .expect("No error should have been generated");
    }

    #[tokio::test]
    async fn test_happy_path_user_repo_exists() {
        let core = Core::builder()
            .with_default_config()
            .with_mock_keychain(|mock| {
                mock.expect_get_token()
                    .with(eq("gh"))
                    .returning(|_| Ok("test_token".into()));
            })
            .with_mock_http_client(mocks::repo_exists("test/user-repo"))
            .build();

        let repo = Repo::new("gh:test/user-repo", std::path::PathBuf::from("/"));
        let service = GitHubService::default();
        service
            .ensure_created(
                &core,
                &Service {
                    name: "gh".into(),
                    website: "https://github.com/{{ .Repo.FullName }}".into(),
                    git_url: "git@github.com/{{ .Repo.FullName }}.git".into(),
                    pattern: "*/*".into(),
                    api: Some(ServiceAPI {
                        kind: "github".into(),
                        url: "https://api.github.com".into(),
                    }),
                },
                &repo,
            )
            .await
            .expect("No error should have been generated");
    }

    #[tokio::test]
    async fn test_is_exists_yes() {
        let core = Core::builder()
            .with_default_config()
            .with_mock_keychain(|mock| {
                mock.expect_get_token()
                    .with(eq("gh"))
                    .returning(|_| Ok("test_token".into()));
            })
            .with_mock_http_client(mocks::get_repo_exists("test/user-repo"))
            .build();

        let repo = Repo::new("gh:test/user-repo", std::path::PathBuf::from("/"));
        let service = GitHubService::default();
        assert!(service
            .is_created(
                &core,
                &Service {
                    name: "gh".into(),
                    website: "https://github.com/{{ .Repo.FullName }}".into(),
                    git_url: "git@github.com/{{ .Repo.FullName }}.git".into(),
                    pattern: "*/*".into(),
                    api: Some(ServiceAPI {
                        kind: "github".into(),
                        url: "https://api.github.com".into(),
                    }),
                },
                &repo,
            )
            .await
            .expect("No error should have been generated"));
    }

    #[tokio::test]
    async fn test_is_exists_no() {
        let core = Core::builder()
            .with_default_config()
            .with_mock_keychain(|mock| {
                mock.expect_get_token()
                    .with(eq("gh"))
                    .returning(|_| Ok("test_token".into()));
            })
            .with_mock_http_client(mocks::get_repo_not_exists("test/user-repo"))
            .build();

        let repo = Repo::new("gh:test/user-repo", std::path::PathBuf::from("/"));
        let service = GitHubService::default();
        assert!(!service
            .is_created(
                &core,
                &Service {
                    name: "gh".into(),
                    website: "https://github.com/{{ .Repo.FullName }}".into(),
                    git_url: "git@github.com/{{ .Repo.FullName }}.git".into(),
                    pattern: "*/*".into(),
                    api: Some(ServiceAPI {
                        kind: "github".into(),
                        url: "https://api.github.com".into(),
                    }),
                },
                &repo,
            )
            .await
            .expect("No error should have been generated"));
    }
}

#[cfg(test)]
pub mod mocks {
    pub fn repo_created(org: &str) -> Vec<super::MockHttpRoute> {
        vec![
            super::MockHttpRoute::new(
                "GET",
                "https://api.github.com/user",
                200,
                r#"{ "login": "test" }"#,
            ),
            super::MockHttpRoute::new(
                "POST",
                "https://api.github.com/user/repos",
                201,
                r#"{ "id": 1234 }"#,
            ),
            super::MockHttpRoute::new(
                "POST",
                format!("https://api.github.com/orgs/{}/repos", org).as_str(),
                201,
                r#"{ "id": 1234 }"#,
            ),
        ]
    }

    pub fn repo_exists(org: &str) -> Vec<super::MockHttpRoute> {
        vec![
            super::MockHttpRoute::new(
                "GET",
                "https://api.github.com/user",
                200,
                r#"{ "login": "test" }"#,
            ),
            super::MockHttpRoute::new(
                "POST",
                "https://api.github.com/user/repos",
                422,
                r#"{"message":"Repository creation failed.","errors":[{"resource":"Repository","code":"custom","field":"name","message":"name already exists on this account"}],"documentation_url":"https://developer.github.com/v3/repos/#create"}"#,
            ),
            super::MockHttpRoute::new(
                "POST",
                format!("https://api.github.com/orgs/{}/repos", org).as_str(),
                422,
                r#"{"message":"Repository creation failed.","errors":[{"resource":"Repository","code":"custom","field":"name","message":"name already exists on this account"}],"documentation_url":"https://developer.github.com/v3/repos/#create"}"#,
            ),
        ]
    }

    pub fn get_repo_exists(repo: &str) -> Vec<super::MockHttpRoute> {
        vec![
            super::MockHttpRoute::new(
                "GET",
                "https://api.github.com/user",
                200,
                r#"{ "login": "test" }"#,
            ),
            super::MockHttpRoute::new(
                "POST",
                "https://api.github.com/user/repos",
                201,
                r#"{ "id": 1234 }"#,
            ),
            super::MockHttpRoute::new(
                "POST",
                format!(
                    "https://api.github.com/orgs/{}/repos",
                    repo.split('/').next().unwrap()
                )
                .as_str(),
                201,
                r#"{ "id": 1234 }"#,
            ),
            super::MockHttpRoute::new(
                "GET",
                format!("https://api.github.com/repos/{}", repo).as_str(),
                200,
                r#"{ "id": 1234 }"#,
            ),
        ]
    }

    pub fn get_repo_not_exists(repo: &str) -> Vec<super::MockHttpRoute> {
        vec![
            super::MockHttpRoute::new(
                "GET",
                "https://api.github.com/user",
                200,
                r#"{ "login": "test" }"#,
            ),
            super::MockHttpRoute::new(
                "POST",
                "https://api.github.com/user/repos",
                201,
                r#"{ "id": 1234 }"#,
            ),
            super::MockHttpRoute::new(
                "POST",
                format!(
                    "https://api.github.com/orgs/{}/repos",
                    repo.split('/').next().unwrap()
                )
                .as_str(),
                201,
                r#"{ "id": 1234 }"#,
            ),
            super::MockHttpRoute::new(
                "GET",
                format!("https://api.github.com/repos/{}", repo).as_str(),
                404,
                r#"{"message":"Not Found","documentation_url":"https://developer.github.com/v3/repos/#get"}"#,
            ),
        ]
    }
}
