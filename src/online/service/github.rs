use super::*;
use crate::errors;
use http::{Request, StatusCode, Uri};
use hyper::Body;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

pub struct GitHubService {}

impl Default for GitHubService {
    fn default() -> Self {
        Self {}
    }
}

#[async_trait]
impl<C: Core> OnlineService<C> for GitHubService {
    fn handles(&self, service: &Service) -> bool {
        service.get_domain() == "github.com"
    }

    async fn ensure_created(&self, core: &C, repo: &Repo) -> Result<(), Error> {
        let current_user = self.get_user_login(core).await?;

        let uri = if repo.get_namespace() == current_user {
            format!("https://api.github.com/user/repos").parse()?
        } else {
            format!("https://api.github.com/orgs/{}/repos", repo.get_namespace()).parse()?
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
                "POST",
                uri,
                Body::from(req_body),
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
    async fn get_user_login<C: Core>(&self, core: &C) -> Result<String, Error> {
        let uri: Uri = "https://api.github.com/user".parse()?;

        let user: Result<UserProfile, GitHubErrorResponse> = self
            .make_request(core, "GET", uri, Body::empty(), vec![StatusCode::OK])
            .await?;

        match user {
            Ok(user) => Ok(user.login),
            Err(e) => Err(e.into()),
        }
    }

    async fn make_request<C: Core, T: DeserializeOwned>(
        &self,
        core: &C,
        method: &str,
        uri: Uri,
        body: Body,
        acceptable: Vec<StatusCode>,
    ) -> Result<Result<T, GitHubErrorResponse>, Error> {
        let token = core.keychain().get_token("github.com")?;

        let req = Request::builder()
            .uri(&uri)
            .method(method)
            .header("User-Agent", version!("Git-Tool/v"))
            .header("Accept", "application/vnd.github.v3+json")
            .header("Authorization", format!("token {}", token))
            .body(body)
            .map_err(|e| {
                errors::system_with_internal(
                    "Unable to construct web request for GitHub.",
                    "Please report this error to us by opening a ticket in GitHub.",
                    e,
                )
            })?;

        let resp = core.http_client().request(req).await?;

        match resp.status() {
            status if acceptable.contains(&status) => {
                let body = hyper::body::to_bytes(resp.into_body()).await?;
                let result = serde_json::from_slice(&body)?;

                Ok(Ok(result))
            }
            status => {
                let body = hyper::body::to_bytes(resp.into_body()).await?;
                let mut result: GitHubErrorResponse = serde_json::from_slice(&body)?;
                result.http_status_code = status;

                Ok(Err(result))
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
struct NewRepoResponse {
    pub id: u64,
}

#[derive(Debug, Deserialize)]
struct GitHubErrorResponse {
    #[serde(skip)]
    pub http_status_code: StatusCode,

    pub message: String,
    pub documentation_url: String,
    #[serde(default)]
    pub errors: Vec<GitHubError>,
}

impl Into<errors::Error> for GitHubErrorResponse {
    fn into(self) -> errors::Error {
        match self.http_status_code {
            http::StatusCode::UNAUTHORIZED => {
                errors::user(
                    "You have not provided a valid authentication token for github.com.",
                    "Please generate a valid Personal Access Token at https://github.com/settings/tokens (with the `repo` scope) and add it using `git-tool auth github.com`.")
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
    use super::mocks::*;
    use super::*;

    #[tokio::test]
    async fn test_happy_path_user_repo() {
        let http = NewRepoSuccessFlow::default();

        let core = CoreBuilder::default()
            .with_mock_keychain(|s| {
                s.set_token("github.com", "test_token").unwrap();
            })
            .with_http_connector(http)
            .build();

        let repo = Repo::new("github.com/test/user-repo", std::path::PathBuf::from("/"));
        let service = GitHubService::default();
        service
            .ensure_created(&core, &repo)
            .await
            .expect("No error should have been generated");
    }

    #[tokio::test]
    async fn test_happy_path_user_repo_exists() {
        let http = NewRepoExistsFlow::default();

        let core = CoreBuilder::default()
            .with_mock_keychain(|s| {
                s.set_token("github.com", "test_token").unwrap();
            })
            .with_http_connector(http)
            .build();

        let repo = Repo::new("github.com/test/user-repo", std::path::PathBuf::from("/"));
        let service = GitHubService::default();
        service
            .ensure_created(&core, &repo)
            .await
            .expect("No error should have been generated");
    }
}

#[cfg(test)]
pub mod mocks {
    pub type NewRepoSuccessFlow = MockGitHubNewRepoSuccessFlow;
    pub type NewRepoExistsFlow = MockGitHubNewRepoDuplicateFlow;

    mock_connector_in_order!(MockGitHubNewRepoSuccessFlow {
r#"HTTP/1.1 200 OK
Content-Type: application/vnd.github.v3+json
Content-Length: 16

{"login":"test"}
"#

r#"HTTP/1.1 201 Created
Content-Type: application/vnd.github.v3+json
Content-Length: 11

{"id":1234}
"#});

    mock_connector_in_order!(MockGitHubNewRepoDuplicateFlow {
r#"HTTP/1.1 200 OK
Content-Type: application/vnd.github.v3+json
Content-Length: 16

{"login":"test"}
"#

r#"HTTP/1.1 422 Unprocessable Entity
Content-Type: application/vnd.github.v3+json
Content-Length: 225

{"message":"Repository creation failed.","errors":[{"resource":"Repository","code":"custom","field":"name","message":"name already exists on this account"}],"documentation_url":"https://developer.github.com/v3/repos/#create"}
"#});
}
