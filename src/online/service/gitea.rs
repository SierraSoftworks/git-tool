use super::*;
use reqwest::{Method, StatusCode};
use serde::Deserialize;
use serde::de::DeserializeOwned;

#[derive(Default)]
pub struct GiteaService {}

#[async_trait]
impl OnlineService for GiteaService {
    fn handles(&self, service: &Service) -> bool {
        service
            .api
            .as_ref()
            .map(|api| api.kind == "Gitea/v1")
            .unwrap_or(false)
    }

    fn auth_instructions(&self) -> String {
        r#"
Create a new Access Token from your account settings (Settings > Applications > Manage Access Tokens).
Configure it with the following permissions:
  - repository: Read and Write
  - organization: Read and Write (only required if you create repositories under an organization)

On gitea.com the token page is available at https://gitea.com/user/settings/applications and on Codeberg at https://codeberg.org/user/settings/applications. For a self-hosted Gitea or Forgejo instance, replace the domain with your instance's URL."#
            .trim()
            .into()
    }

    async fn test(&self, core: &Core, service: &Service) -> Result<(), human_errors::Error> {
        self.get_current_username(core, service).await?;
        Ok(())
    }

    #[tracing::instrument(err, skip(self, core))]
    async fn is_created(
        &self,
        core: &Core,
        service: &Service,
        repo: &Repo,
    ) -> Result<bool, human_errors::Error> {
        let uri = format!("{}/repos/{}", self.api_url(service), repo.get_full_name());

        let resp: Result<RepoResponse, GiteaErrorResponse> = self
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
        let current_user = self.get_current_username(core, service).await?;

        let uri = if repo.namespace.eq_ignore_ascii_case(&current_user) {
            format!("{}/user/repos", self.api_url(service))
        } else {
            format!("{}/orgs/{}/repos", self.api_url(service), &repo.namespace)
        };

        let body = serde_json::to_vec(&json!({
            "name": repo.get_name(),
            "private": core
                .config()
                .get_features()
                .has(features::CREATE_REMOTE_PRIVATE),
        }))
        .wrap_system_err(
            "Failed to serialize repository information for submission to Gitea as part of repo creation.", &[
                "Please report this issue to us by creating a new GitHub issue.",
                "Try creating your repository with `git-tool new --no-create-remote` and then pushing it to your Gitea server manually."
            ])?;

        let resp: Result<RepoResponse, GiteaErrorResponse> = self
            .make_request(
                core,
                service,
                Method::POST,
                &uri,
                body,
                vec![StatusCode::CREATED],
            )
            .await?;

        match resp {
            Ok(_) => Ok(()),
            Err(e) if e.http_status_code == StatusCode::CONFLICT => Ok(()),
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
        if source
            .namespace
            .eq_ignore_ascii_case(&destination.namespace)
        {
            self.rename_repo(
                core,
                service,
                &source.namespace,
                &source.name,
                &destination.name,
            )
            .await?;
        } else {
            // Gitea's transfer endpoint moves the repository to a new owner but keeps its
            // name, so we perform the transfer first and then rename the repository if the
            // caller also asked for a new name.
            let uri = format!(
                "{}/repos/{}/{}/transfer",
                self.api_url(service),
                source.namespace,
                source.name
            );

            let body = serde_json::to_vec(&json!({
                "new_owner": destination.namespace,
            }))
            .wrap_system_err(
                "Failed to serialize repository information for submission to Gitea as part of repo transfer.", &[
                    "Please report this issue to us by creating a new GitHub issue.",
                    "Try transferring the repository manually using your Gitea server's repository settings page."
                ])?;

            let resp: Result<RepoResponse, GiteaErrorResponse> = self
                .make_request(
                    core,
                    service,
                    Method::POST,
                    &uri,
                    body,
                    vec![StatusCode::ACCEPTED, StatusCode::CREATED, StatusCode::OK],
                )
                .await?;

            if let Err(e) = resp {
                return Err(e.into());
            }

            if !source.name.eq_ignore_ascii_case(&destination.name) {
                self.rename_repo(
                    core,
                    service,
                    &destination.namespace,
                    &source.name,
                    &destination.name,
                )
                .await?;
            }
        }

        Ok(())
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
            "{}/repos/{}/{}/forks",
            self.api_url(service),
            source.namespace,
            source.name
        );

        let current_user = self.get_current_username(core, service).await?;

        let mut body_json = json!({
            "name": destination.name,
        });

        if !destination.namespace.eq_ignore_ascii_case(&current_user)
            && let serde_json::Value::Object(map) = &mut body_json
        {
            map.insert(
                "organization".into(),
                serde_json::Value::String(destination.namespace.clone()),
            );
        }

        let body = serde_json::to_vec(&body_json).wrap_system_err(
            "Failed to serialize repository information for submission to Gitea as part of repo fork.", &[
                "Please report this issue to us by creating a new GitHub issue.",
                "Try forking the repository manually using your Gitea server's repository page."
            ])?;

        let resp: Result<RepoResponse, GiteaErrorResponse> = self
            .make_request(
                core,
                service,
                Method::POST,
                &uri,
                body,
                vec![StatusCode::ACCEPTED, StatusCode::CREATED, StatusCode::OK],
            )
            .await?;

        match resp {
            Ok(_) => Ok(()),
            Err(e) => Err(e.into()),
        }
    }
}

impl GiteaService {
    fn api_url(&self, service: &Service) -> String {
        service
            .api
            .as_ref()
            .map(|api| api.url.trim_end_matches('/').to_string())
            .unwrap_or_default()
    }

    async fn get_current_username(
        &self,
        core: &Core,
        service: &Service,
    ) -> Result<String, human_errors::Error> {
        let uri = format!("{}/user", self.api_url(service));
        let user: Result<UserProfile, GiteaErrorResponse> = self
            .make_request(
                core,
                service,
                Method::GET,
                &uri,
                Vec::new(),
                vec![StatusCode::OK],
            )
            .await?;

        match user {
            Ok(user) => Ok(user.login),
            Err(e) => Err(e.into()),
        }
    }

    async fn rename_repo(
        &self,
        core: &Core,
        service: &Service,
        owner: &str,
        name: &str,
        new_name: &str,
    ) -> Result<(), human_errors::Error> {
        let uri = format!("{}/repos/{}/{}", self.api_url(service), owner, name);

        let body = serde_json::to_vec(&json!({
            "name": new_name,
        }))
        .wrap_system_err(
            "Failed to serialize repository information for submission to Gitea as part of repo rename.", &[
                "Please report this issue to us by creating a new GitHub issue.",
                "Try renaming the repository manually using your Gitea server's repository settings page."
            ])?;

        let resp: Result<RepoResponse, GiteaErrorResponse> = self
            .make_request(
                core,
                service,
                Method::PATCH,
                &uri,
                body,
                vec![StatusCode::OK],
            )
            .await?;

        match resp {
            Ok(_) => Ok(()),
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
    ) -> Result<Result<T, GiteaErrorResponse>, human_errors::Error> {
        let token = core.keychain().get_token(&service.name)?;

        let headers = vec![
            ("Accept", "application/json".to_string()),
            ("Content-Type", "application/json".to_string()),
            ("Authorization", format!("token {token}")),
        ];

        let (status, bytes) = request_with_retry(core, method, uri, &headers, body).await?;

        if acceptable.contains(&status) {
            let result = serde_json::from_slice(&bytes).wrap_system_err(
                "We could not deserialize the response from Gitea because it didn't match the expected response format.",
                &["Please report this issue to us on GitHub with the trace ID for the command you were running so that we can investigate."],
            )?;

            return Ok(Ok(result));
        }

        let mut result: GiteaErrorResponse = serde_json::from_slice(&bytes).unwrap_or_default();
        result.http_status_code = status;
        Ok(Err(result))
    }
}

#[derive(Debug, Deserialize)]
struct UserProfile {
    pub login: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct RepoResponse {
    pub id: u64,
}

#[derive(Debug, Default, Deserialize)]
#[allow(dead_code)]
struct GiteaErrorResponse {
    #[serde(skip)]
    pub http_status_code: StatusCode,

    #[serde(default)]
    pub message: String,

    #[serde(default)]
    pub url: String,
}

#[allow(clippy::from_over_into)]
impl Into<Error> for GiteaErrorResponse {
    fn into(self) -> Error {
        match self.http_status_code {
            StatusCode::UNAUTHORIZED => human_errors::user(
                "You have not provided a valid authentication token for your Gitea server.",
                &[
                    "Generate a valid Access Token with the `repository` scope from your account's applications settings and add it using `git-tool auth <service>`.",
                ],
            ),
            StatusCode::FORBIDDEN => human_errors::wrap_user(
                format!("{self:?}"),
                format!(
                    "You do not have permission to perform this action on your Gitea server: {}",
                    self.message
                ),
                &[
                    "Check that your access token has the `repository` (and, for organizations, `organization`) scope and that you have permission to manage this repository.",
                ],
            ),
            StatusCode::NOT_FOUND => human_errors::wrap_user(
                format!("{self:?}"),
                "We could not find the Gitea organization or user you specified.",
                &[
                    "Check that you have specified the correct organization or user in the repository name and try again.",
                ],
            ),
            StatusCode::TOO_MANY_REQUESTS => human_errors::user(
                "Your Gitea server has rate limited requests from your IP address.",
                &["Please wait until the rate limit is removed before trying again."],
            ),
            status => human_errors::wrap_system(
                format!("{self:?}"),
                format!(
                    "Received an HTTP {} {} response from Gitea: {}",
                    status.as_u16(),
                    status.canonical_reason().unwrap_or_default(),
                    self.message
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
            name: "gitea".into(),
            website: "https://gitea.com/{{ .Repo.FullName }}".into(),
            git_url: "git@gitea.com:{{ .Repo.FullName }}.git".into(),
            pattern: "*/*".into(),
            api: Some(ServiceAPI {
                kind: "Gitea/v1".into(),
                url: "https://gitea.com/api/v1".into(),
            }),
        }
    }

    fn core(mocks: Vec<MockHttpRoute>) -> Core {
        Core::builder()
            .with_default_config()
            .with_mock_keychain(|mock| {
                mock.expect_get_token()
                    .with(eq("gitea"))
                    .returning(|_| Ok("test_token".into()));
            })
            .with_mock_http_client(mocks)
            .build()
    }

    #[tokio::test]
    async fn test_create_user_repo() {
        let core = core(vec![
            MockHttpRoute::new(
                "GET",
                "https://gitea.com/api/v1/user",
                200,
                r#"{ "login": "test" }"#,
            ),
            MockHttpRoute::new(
                "POST",
                "https://gitea.com/api/v1/user/repos",
                201,
                r#"{ "id": 1234 }"#,
            ),
        ]);

        let repo = Repo::new("gitea:test/user-repo", std::path::PathBuf::from("/"));
        GiteaService::default()
            .ensure_created(&core, &service(), &repo)
            .await
            .expect("No error should have been generated");
    }

    #[tokio::test]
    async fn test_create_org_repo() {
        let core = core(vec![
            MockHttpRoute::new(
                "GET",
                "https://gitea.com/api/v1/user",
                200,
                r#"{ "login": "test" }"#,
            ),
            MockHttpRoute::new(
                "POST",
                "https://gitea.com/api/v1/orgs/myorg/repos",
                201,
                r#"{ "id": 1234 }"#,
            ),
        ]);

        let repo = Repo::new("gitea:myorg/user-repo", std::path::PathBuf::from("/"));
        GiteaService::default()
            .ensure_created(&core, &service(), &repo)
            .await
            .expect("No error should have been generated");
    }

    #[tokio::test]
    async fn test_create_repo_exists() {
        let core = core(vec![
            MockHttpRoute::new(
                "GET",
                "https://gitea.com/api/v1/user",
                200,
                r#"{ "login": "test" }"#,
            ),
            MockHttpRoute::new(
                "POST",
                "https://gitea.com/api/v1/user/repos",
                409,
                r#"{ "message": "The repository with the same name already exists." }"#,
            ),
        ]);

        let repo = Repo::new("gitea:test/user-repo", std::path::PathBuf::from("/"));
        GiteaService::default()
            .ensure_created(&core, &service(), &repo)
            .await
            .expect("No error should have been generated");
    }

    #[tokio::test]
    async fn test_is_created_yes() {
        let core = core(vec![MockHttpRoute::new(
            "GET",
            "https://gitea.com/api/v1/repos/test/user-repo",
            200,
            r#"{ "id": 1234 }"#,
        )]);

        let repo = Repo::new("gitea:test/user-repo", std::path::PathBuf::from("/"));
        assert!(
            GiteaService::default()
                .is_created(&core, &service(), &repo)
                .await
                .expect("No error should have been generated")
        );
    }

    #[tokio::test]
    async fn test_is_created_no() {
        let core = core(vec![MockHttpRoute::new(
            "GET",
            "https://gitea.com/api/v1/repos/test/user-repo",
            404,
            r#"{ "message": "Not Found" }"#,
        )]);

        let repo = Repo::new("gitea:test/user-repo", std::path::PathBuf::from("/"));
        assert!(
            !GiteaService::default()
                .is_created(&core, &service(), &repo)
                .await
                .expect("No error should have been generated")
        );
    }

    #[tokio::test]
    async fn test_move_same_namespace() {
        let core = core(vec![MockHttpRoute::new(
            "PATCH",
            "https://gitea.com/api/v1/repos/test/user-repo",
            200,
            r#"{ "id": 1234 }"#,
        )]);

        let src = Repo::new("gitea:test/user-repo", std::path::PathBuf::from("/"));
        let dest = Repo::new("gitea:test/new-name", std::path::PathBuf::from("/"));
        GiteaService::default()
            .move_repo(&core, &service(), &src, &dest)
            .await
            .expect("No error should have been generated");
    }

    #[tokio::test]
    async fn test_move_different_namespace() {
        let core = core(vec![
            MockHttpRoute::new(
                "POST",
                "https://gitea.com/api/v1/repos/test/user-repo/transfer",
                202,
                r#"{ "id": 1234 }"#,
            ),
            MockHttpRoute::new(
                "PATCH",
                "https://gitea.com/api/v1/repos/other/user-repo",
                200,
                r#"{ "id": 1234 }"#,
            ),
        ]);

        let src = Repo::new("gitea:test/user-repo", std::path::PathBuf::from("/"));
        let dest = Repo::new("gitea:other/new-name", std::path::PathBuf::from("/"));
        GiteaService::default()
            .move_repo(&core, &service(), &src, &dest)
            .await
            .expect("No error should have been generated");
    }

    #[tokio::test]
    async fn test_fork_repo() {
        let core = core(vec![
            MockHttpRoute::new(
                "GET",
                "https://gitea.com/api/v1/user",
                200,
                r#"{ "login": "test" }"#,
            ),
            MockHttpRoute::new(
                "POST",
                "https://gitea.com/api/v1/repos/other/user-repo/forks",
                202,
                r#"{ "id": 1234 }"#,
            ),
        ]);

        let src = Repo::new("gitea:other/user-repo", std::path::PathBuf::from("/"));
        let dest = Repo::new("gitea:test/user-repo", std::path::PathBuf::from("/"));
        GiteaService::default()
            .fork_repo(&core, &service(), &src, &dest, true)
            .await
            .expect("No error should have been generated");
    }
}
