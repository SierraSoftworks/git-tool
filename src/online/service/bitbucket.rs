use super::*;
use base64::prelude::{BASE64_STANDARD, Engine};
use reqwest::{Method, StatusCode};
use serde::Deserialize;
use serde::de::DeserializeOwned;

#[derive(Default)]
pub struct BitBucketService {}

#[async_trait]
impl OnlineService for BitBucketService {
    fn handles(&self, service: &Service) -> bool {
        service
            .api
            .as_ref()
            .map(|api| api.kind == "BitBucket/2.0")
            .unwrap_or(false)
    }

    fn auth_instructions(&self) -> String {
        r#"
BitBucket Cloud supports two kinds of credentials which you can use with Git-Tool:

  1. App Passwords (recommended): create one at https://bitbucket.org/account/settings/app-passwords/
     with the "Repositories: Read, Write, Admin" permissions, then enter it in the form
     `username:app_password` (using your BitBucket username).

  2. Access Tokens: create a Workspace, Project or Repository Access Token with the
     "repository:admin" scope and paste the token on its own.

Git-Tool will automatically use HTTP Basic authentication when you provide a `username:app_password`
value and Bearer authentication when you provide a standalone access token."#
            .trim()
            .into()
    }

    async fn test(&self, core: &Core, service: &Service) -> Result<(), human_errors::Error> {
        let uri = format!("{}/user", self.api_url(service));
        let resp: Result<UserProfile, BitBucketErrorResponse> = self
            .make_request(
                core,
                service,
                Method::GET,
                &uri,
                Vec::new(),
                vec![StatusCode::OK],
            )
            .await?;

        match resp {
            Ok(_) => Ok(()),
            Err(e) => Err(e.into()),
        }
    }

    #[tracing::instrument(err, skip(self, core))]
    async fn is_created(
        &self,
        core: &Core,
        service: &Service,
        repo: &Repo,
    ) -> Result<bool, human_errors::Error> {
        let uri = format!(
            "{}/repositories/{}/{}",
            self.api_url(service),
            repo.namespace,
            repo.name
        );

        let resp: Result<RepoResponse, BitBucketErrorResponse> = self
            .make_request(
                core,
                service,
                Method::GET,
                &uri,
                Vec::new(),
                vec![StatusCode::OK],
            )
            .await?;

        match resp {
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
    ) -> Result<(), human_errors::Error> {
        let uri = format!(
            "{}/repositories/{}/{}",
            self.api_url(service),
            repo.namespace,
            repo.name
        );

        let body = serde_json::to_vec(&json!({
            "scm": "git",
            "is_private": core
                .config()
                .get_features()
                .has(features::CREATE_REMOTE_PRIVATE),
        }))
        .wrap_system_err(
            "Failed to serialize repository information for submission to BitBucket as part of repo creation.", &[
                "Please report this issue to us by creating a new GitHub issue.",
                "Try creating your repository with `git-tool new --no-create-remote` and then pushing it to BitBucket manually."
            ])?;

        let resp: Result<RepoResponse, BitBucketErrorResponse> = self
            .make_request(
                core,
                service,
                Method::POST,
                &uri,
                body,
                vec![StatusCode::OK, StatusCode::CREATED],
            )
            .await?;

        match resp {
            Ok(_) => Ok(()),
            Err(e) if e.already_exists() => Ok(()),
            Err(e) => Err(e.into()),
        }
    }

    #[tracing::instrument(err, skip(self, core))]
    async fn move_repo(
        &self,
        core: &Core,
        service: &Service,
        source: &Repo,
        destination: &Repo,
    ) -> Result<(), human_errors::Error> {
        // BitBucket Cloud's REST API only supports renaming a repository within the same
        // workspace; there is no supported endpoint for transferring a repository between
        // workspaces, so we surface a helpful error in that case.
        if !source
            .namespace
            .eq_ignore_ascii_case(&destination.namespace)
        {
            return Err(human_errors::user(
                format!(
                    "BitBucket does not support moving the repository '{}' to a different workspace ('{}') through its API.",
                    source.get_full_name(),
                    destination.namespace
                ),
                &[
                    "Move the repository to the new workspace manually using BitBucket's repository settings page, or keep it within the same workspace when renaming it.",
                ],
            ));
        }

        let uri = format!(
            "{}/repositories/{}/{}",
            self.api_url(service),
            source.namespace,
            source.name
        );

        let body = serde_json::to_vec(&json!({
            "name": destination.name,
        }))
        .wrap_system_err(
            "Failed to serialize repository information for submission to BitBucket as part of repo rename.", &[
                "Please report this issue to us by creating a new GitHub issue.",
                "Try renaming the repository manually on BitBucket using the repository settings page."
            ])?;

        let resp: Result<RepoResponse, BitBucketErrorResponse> = self
            .make_request(core, service, Method::PUT, &uri, body, vec![StatusCode::OK])
            .await?;

        match resp {
            Ok(_) => Ok(()),
            Err(e) => Err(e.into()),
        }
    }

    #[tracing::instrument(err, skip(self, core))]
    async fn fork_repo(
        &self,
        core: &Core,
        service: &Service,
        source: &Repo,
        destination: &Repo,
        _default_branch_only: bool,
    ) -> Result<(), human_errors::Error> {
        let uri = format!(
            "{}/repositories/{}/{}/forks",
            self.api_url(service),
            source.namespace,
            source.name
        );

        let body = serde_json::to_vec(&json!({
            "name": destination.name,
            "workspace": {
                "slug": destination.namespace,
            },
        }))
        .wrap_system_err(
            "Failed to serialize repository information for submission to BitBucket as part of repo fork.", &[
                "Please report this issue to us by creating a new GitHub issue.",
                "Try forking the repository manually on BitBucket using the repository page."
            ])?;

        let resp: Result<RepoResponse, BitBucketErrorResponse> = self
            .make_request(
                core,
                service,
                Method::POST,
                &uri,
                body,
                vec![StatusCode::OK, StatusCode::CREATED],
            )
            .await?;

        match resp {
            Ok(_) => Ok(()),
            Err(e) => Err(e.into()),
        }
    }
}

impl BitBucketService {
    fn api_url(&self, service: &Service) -> String {
        service
            .api
            .as_ref()
            .map(|api| api.url.trim_end_matches('/').to_string())
            .unwrap_or_default()
    }

    fn authorization_header(token: &str) -> String {
        // App Passwords and API tokens are provided as `username:secret` and use HTTP Basic
        // authentication, while standalone Access Tokens use Bearer authentication.
        if token.contains(':') {
            format!("Basic {}", BASE64_STANDARD.encode(token.as_bytes()))
        } else {
            format!("Bearer {token}")
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
    ) -> Result<Result<T, BitBucketErrorResponse>, human_errors::Error> {
        let token = core.keychain().get_token(&service.name)?;

        let headers = vec![
            ("Accept", "application/json".to_string()),
            ("Content-Type", "application/json".to_string()),
            ("Authorization", Self::authorization_header(&token)),
        ];

        let (status, bytes) = request_with_retry(core, method, uri, &headers, body).await?;

        if acceptable.contains(&status) {
            let result = serde_json::from_slice(&bytes).wrap_system_err(
                "We could not deserialize the response from BitBucket because it didn't match the expected response format.",
                &["Please report this issue to us on GitHub with the trace ID for the command you were running so that we can investigate."],
            )?;

            return Ok(Ok(result));
        }

        let mut result: BitBucketErrorResponse = serde_json::from_slice(&bytes).unwrap_or_default();
        result.http_status_code = status;
        Ok(Err(result))
    }
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct UserProfile {
    #[serde(default)]
    pub username: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct RepoResponse {
    #[serde(default)]
    pub full_name: String,
}

#[derive(Debug, Default, Deserialize)]
struct BitBucketErrorResponse {
    #[serde(skip)]
    pub http_status_code: StatusCode,

    #[serde(default)]
    pub error: BitBucketErrorBody,
}

#[derive(Debug, Default, Deserialize)]
#[allow(dead_code)]
struct BitBucketErrorBody {
    #[serde(default)]
    pub message: String,

    #[serde(default)]
    pub detail: String,
}

impl BitBucketErrorResponse {
    fn already_exists(&self) -> bool {
        self.http_status_code == StatusCode::BAD_REQUEST
            && self.error.message.to_lowercase().contains("already exists")
    }
}

#[allow(clippy::from_over_into)]
impl Into<Error> for BitBucketErrorResponse {
    fn into(self) -> Error {
        match self.http_status_code {
            StatusCode::UNAUTHORIZED => human_errors::user(
                "You have not provided a valid authentication token for BitBucket.",
                &[
                    "Create an App Password (entered as `username:app_password`) or an Access Token and add it using `git-tool auth <service>`.",
                ],
            ),
            StatusCode::FORBIDDEN => human_errors::wrap_user(
                format!("{self:?}"),
                format!(
                    "You do not have permission to perform this action on BitBucket: {}",
                    self.error.message
                ),
                &[
                    "Check that your App Password or Access Token includes the repository administration permissions and that you have access to this workspace.",
                ],
            ),
            StatusCode::NOT_FOUND => human_errors::wrap_user(
                format!("{self:?}"),
                "We could not find the BitBucket workspace or repository you specified.",
                &[
                    "Check that you have specified the correct workspace in the repository name and try again.",
                ],
            ),
            StatusCode::TOO_MANY_REQUESTS => human_errors::user(
                "BitBucket has rate limited requests from your IP address.",
                &["Please wait until BitBucket removes this rate limit before trying again."],
            ),
            status => human_errors::wrap_system(
                format!("{self:?}"),
                format!(
                    "Received an HTTP {} {} response from BitBucket: {}",
                    status.as_u16(),
                    status.canonical_reason().unwrap_or_default(),
                    self.error.message
                ),
                &[
                    "Please read the error message below and decide if there is something you can do to fix the problem, or report it to us on GitHub.",
                ],
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::eq;

    fn service() -> Service {
        Service {
            name: "bitbucket".into(),
            website: "https://bitbucket.org/{{ .Repo.FullName }}".into(),
            git_url: "git@bitbucket.org:{{ .Repo.FullName }}.git".into(),
            pattern: "*/*".into(),
            api: Some(ServiceAPI {
                kind: "BitBucket/2.0".into(),
                url: "https://api.bitbucket.org/2.0".into(),
            }),
        }
    }

    fn core(mocks: Vec<MockHttpRoute>) -> Core {
        Core::builder()
            .with_default_config()
            .with_mock_keychain(|mock| {
                mock.expect_get_token()
                    .with(eq("bitbucket"))
                    .returning(|_| Ok("user:app_password".into()));
            })
            .with_mock_http_client(mocks)
            .build()
    }

    #[test]
    fn test_authorization_header() {
        assert_eq!(
            BitBucketService::authorization_header("user:app_password"),
            format!("Basic {}", BASE64_STANDARD.encode("user:app_password"))
        );
        assert_eq!(
            BitBucketService::authorization_header("access-token"),
            "Bearer access-token"
        );
    }

    #[tokio::test]
    async fn test_create_repo() {
        let core = core(vec![MockHttpRoute::new(
            "POST",
            "https://api.bitbucket.org/2.0/repositories/myworkspace/user-repo",
            200,
            r#"{ "full_name": "myworkspace/user-repo" }"#,
        )]);

        let repo = Repo::new(
            "bitbucket:myworkspace/user-repo",
            std::path::PathBuf::from("/"),
        );
        BitBucketService::default()
            .ensure_created(&core, &service(), &repo)
            .await
            .expect("No error should have been generated");
    }

    #[tokio::test]
    async fn test_create_repo_exists() {
        let core = core(vec![MockHttpRoute::new(
            "POST",
            "https://api.bitbucket.org/2.0/repositories/myworkspace/user-repo",
            400,
            r#"{ "type": "error", "error": { "message": "Repository with this Slug and Owner already exists." } }"#,
        )]);

        let repo = Repo::new(
            "bitbucket:myworkspace/user-repo",
            std::path::PathBuf::from("/"),
        );
        BitBucketService::default()
            .ensure_created(&core, &service(), &repo)
            .await
            .expect("No error should have been generated");
    }

    #[tokio::test]
    async fn test_is_created_yes() {
        let core = core(vec![MockHttpRoute::new(
            "GET",
            "https://api.bitbucket.org/2.0/repositories/myworkspace/user-repo",
            200,
            r#"{ "full_name": "myworkspace/user-repo" }"#,
        )]);

        let repo = Repo::new(
            "bitbucket:myworkspace/user-repo",
            std::path::PathBuf::from("/"),
        );
        assert!(
            BitBucketService::default()
                .is_created(&core, &service(), &repo)
                .await
                .expect("No error should have been generated")
        );
    }

    #[tokio::test]
    async fn test_is_created_no() {
        let core = core(vec![MockHttpRoute::new(
            "GET",
            "https://api.bitbucket.org/2.0/repositories/myworkspace/user-repo",
            404,
            r#"{ "type": "error", "error": { "message": "Repository not found" } }"#,
        )]);

        let repo = Repo::new(
            "bitbucket:myworkspace/user-repo",
            std::path::PathBuf::from("/"),
        );
        assert!(
            !BitBucketService::default()
                .is_created(&core, &service(), &repo)
                .await
                .expect("No error should have been generated")
        );
    }

    #[tokio::test]
    async fn test_move_same_workspace() {
        let core = core(vec![MockHttpRoute::new(
            "PUT",
            "https://api.bitbucket.org/2.0/repositories/myworkspace/user-repo",
            200,
            r#"{ "full_name": "myworkspace/new-name" }"#,
        )]);

        let src = Repo::new(
            "bitbucket:myworkspace/user-repo",
            std::path::PathBuf::from("/"),
        );
        let dest = Repo::new(
            "bitbucket:myworkspace/new-name",
            std::path::PathBuf::from("/"),
        );
        BitBucketService::default()
            .move_repo(&core, &service(), &src, &dest)
            .await
            .expect("No error should have been generated");
    }

    #[tokio::test]
    async fn test_move_different_workspace_errors() {
        let core = core(vec![]);

        let src = Repo::new(
            "bitbucket:myworkspace/user-repo",
            std::path::PathBuf::from("/"),
        );
        let dest = Repo::new("bitbucket:other/new-name", std::path::PathBuf::from("/"));
        BitBucketService::default()
            .move_repo(&core, &service(), &src, &dest)
            .await
            .expect_err("An error should have been generated for a cross-workspace move");
    }

    #[tokio::test]
    async fn test_fork_repo() {
        let core = core(vec![MockHttpRoute::new(
            "POST",
            "https://api.bitbucket.org/2.0/repositories/other/user-repo/forks",
            201,
            r#"{ "full_name": "myworkspace/user-repo" }"#,
        )]);

        let src = Repo::new("bitbucket:other/user-repo", std::path::PathBuf::from("/"));
        let dest = Repo::new(
            "bitbucket:myworkspace/user-repo",
            std::path::PathBuf::from("/"),
        );
        BitBucketService::default()
            .fork_repo(&core, &service(), &src, &dest, true)
            .await
            .expect("No error should have been generated");
    }
}
