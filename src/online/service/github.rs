use super::*;
use crate::errors;
use reqwest::{Method, Request, StatusCode};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

pub struct GitHubService {}

impl Default for GitHubService {
    fn default() -> Self {
        Self {}
    }
}

#[async_trait]
impl OnlineService for GitHubService {
    fn handles(&self, service: &Service) -> bool {
        service.get_domain() == "github.com"
    }

    async fn ensure_created(&self, core: &Core, repo: &Repo) -> Result<(), Error> {
        let current_user = self.get_user_login(core).await?;

        let uri = if repo.get_namespace() == current_user {
            format!("https://api.github.com/user/repos")
        } else {
            format!("https://api.github.com/orgs/{}/repos", repo.get_namespace())
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
    async fn get_user_login(&self, core: &Core) -> Result<String, Error> {
        let user: Result<UserProfile, GitHubErrorResponse> = self
            .make_request(
                core,
                Method::GET,
                "https://api.github.com/user",
                "",
                vec![StatusCode::OK],
            )
            .await?;

        match user {
            Ok(user) => Ok(user.login),
            Err(e) => Err(e.into()),
        }
    }

    async fn make_request<B: Into<reqwest::Body>, T: DeserializeOwned>(
        &self,
        core: &Core,
        method: Method,
        uri: &str,
        body: B,
        acceptable: Vec<StatusCode>,
    ) -> Result<Result<T, GitHubErrorResponse>, Error> {
        let url = uri.parse().map_err(|e| {
            errors::system_with_internal(
                &format!("Unable to parse GitHub API URL '{}'.", uri),
                "Please report this error to us by opening an issue on GitHub.",
                e,
            )
        })?;

        let token = core.keychain().get_token("github.com")?;

        let mut req = Request::new(method, url);

        *req.body_mut() = Some(body.into());

        let headers = req.headers_mut();

        headers.append("User-Agent", version!("Git-Tool/v").parse()?);
        headers.append("Accept", "application/vnd.github.v3+json".parse()?);
        headers.append("Authorization", format!("token {}", token).parse()?);

        let resp = core.http_client().request(req).await?;

        match resp.status() {
            status if acceptable.contains(&status) => {
                let result = resp.json().await?;

                Ok(Ok(result))
            }
            status => {
                let mut result: GitHubErrorResponse = resp.json().await?;
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
    use mocktopus::mocking::*;

    #[tokio::test]
    async fn test_happy_path_user_repo() {
        super::KeyChain::get_token.mock_safe(|_, token| {
            assert_eq!(token, "github.com", "the correct token should be requested");
            MockResult::Return(Ok("test_token".into()))
        });

        mocks::repo_created("test");

        let core = Core::builder().build();

        let repo = Repo::new("github.com/test/user-repo", std::path::PathBuf::from("/"));
        let service = GitHubService::default();
        service
            .ensure_created(&core, &repo)
            .await
            .expect("No error should have been generated");
    }

    #[tokio::test]
    async fn test_happy_path_user_repo_exists() {
        super::KeyChain::get_token.mock_safe(|_, token| {
            assert_eq!(token, "github.com", "the correct token should be requested");
            MockResult::Return(Ok("test_token".into()))
        });

        mocks::repo_exists("test");

        let core = Core::builder().build();

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
    pub fn repo_created(org: &str) {
        super::HttpClient::mock(vec![
            super::HttpClient::route(
                "GET",
                "https://api.github.com/user",
                200,
                r#"{ "login": "test" }"#,
            ),
            super::HttpClient::route(
                "POST",
                "https://api.github.com/user/repos",
                201,
                r#"{ "id": 1234 }"#,
            ),
            super::HttpClient::route(
                "POST",
                format!("https://api.github.com/orgs/{}/repos", org).as_str(),
                201,
                r#"{ "id": 1234 }"#,
            ),
        ]);
    }

    pub fn repo_exists(org: &str) {
        super::HttpClient::mock(vec![
            super::HttpClient::route(
                "GET",
                "https://api.github.com/user",
                200,
                r#"{ "login": "test" }"#,
            ),
            super::HttpClient::route(
                "POST",
                "https://api.github.com/user/repos",
                422,
                r#"{"message":"Repository creation failed.","errors":[{"resource":"Repository","code":"custom","field":"name","message":"name already exists on this account"}],"documentation_url":"https://developer.github.com/v3/repos/#create"}"#,
            ),
            super::HttpClient::route(
                "POST",
                format!("https://api.github.com/orgs/{}/repos", org).as_str(),
                422,
                r#"{"message":"Repository creation failed.","errors":[{"resource":"Repository","code":"custom","field":"name","message":"name already exists on this account"}],"documentation_url":"https://developer.github.com/v3/repos/#create"}"#,
            ),
        ]);
    }
}
