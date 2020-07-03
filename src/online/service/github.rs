use super::*;
use hyper::{Body, Client};
use http::{Request, Uri, StatusCode};
use serde::{Serialize, Deserialize};
use serde::de::DeserializeOwned;
use crate::errors;

pub struct GitHubService {
    client: Client<hyper_tls::HttpsConnector<hyper::client::HttpConnector>, Body>,
}

impl Default for GitHubService {
    fn default() -> Self {
        let https = hyper_tls::HttpsConnector::new();
        let client = Client::builder()
            .build(https);

        Self {
            client,
        }
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
            private: core.config().get_features().create_remote_private()
        };

        let req_body = serde_json::to_vec(&new_repo)?;
        self.make_request(core, "POST", uri, Body::from(req_body)).await?;

        Ok(())
    }
}

impl GitHubService {
    async fn get_user_login<C: Core>(&self, core: &C) -> Result<String, Error> {
        let uri: Uri = "https://api.github.com/user".parse()?;

        let user: UserProfile = self.make_request(core, "GET", uri, Body::empty()).await?;

        Ok(user.login)
    }

    async fn make_request<C: Core, T: DeserializeOwned>(&self, core: &C, method: &str, uri: Uri, body: Body) -> Result<T, Error> {
        let token = core.keychain().get_token("github.com")?;

        let req = Request::builder()
            .uri(uri)
            .method(method)
            .header("User-Agent", format!("Git-Tool/{}", env!("CARGO_PKG_VERSION")))
            .header("Accept", "application/vnd.github.v3+json")
            .header("Authorization", format!("token {}", token))
            .body(body)
            .map_err(|e| errors::system_with_internal(
                "Unable to construct web request for GitHub.",
                "Please report this error to us by opening a ticket in GitHub.",
                e))?;

        let resp = self.client.request(req).await?;

        match resp.status() {
            StatusCode::OK | StatusCode::CREATED => {
                let body = hyper::body::to_bytes(resp.into_body()).await?;
                let result = serde_json::from_slice(&body)?;

                Ok(result)
            },
            http::StatusCode::UNAUTHORIZED => {
                Err(errors::user(
                    "You have not provided a valid authentication token for github.com.",
                    "Please generate a valid Personal Access Token at https://github.com/settings/tokens (with the `repo` scope) and add it using `git-tool auth github.com`."))
            },
            http::StatusCode::TOO_MANY_REQUESTS => {
                Err(errors::user(
                    "GitHub has rate limited requests from your IP address.",
                    "Please wait until GitHub removes this rate limit before trying again."))
            },
            status => {
                let inner_error = errors::hyper::HyperResponseError::with_body(resp).await;
                Err(errors::system_with_internal(
                    &format!("Received an HTTP {} {} response from GitHub when.", status.as_u16(), status.canonical_reason().unwrap_or_default()),
                    "Please read the error message below and decide if there is something you can do to fix the problem, or report it to us on GitHub.",
                    inner_error))
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct NewRepo {
    pub name: String,
    pub private: bool
}

#[derive(Debug, Deserialize)]
struct UserProfile {
    pub login: String,
}