use super::*;
use percent_encoding::{AsciiSet, NON_ALPHANUMERIC, utf8_percent_encode};
use reqwest::{Method, StatusCode};
use serde::Deserialize;
use serde::de::DeserializeOwned;
use serde_json::Value;

#[derive(Default)]
pub struct GitLabService {}

#[async_trait]
impl OnlineService for GitLabService {
    fn handles(&self, service: &Service) -> bool {
        service
            .api
            .as_ref()
            .map(|api| api.kind == "GitLab/v4")
            .unwrap_or(false)
    }

    fn auth_instructions(&self) -> String {
        r#"
Create a new Personal Access Token with the 'api' scope at https://gitlab.com/-/user_settings/personal_access_tokens
Configure it with the following:
  - Scopes: api

For a self-managed GitLab instance, replace the domain above with your instance's URL (for example https://gitlab.example.com/-/user_settings/personal_access_tokens)."#
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
        let uri = format!(
            "{}/projects/{}",
            self.api_url(service),
            encode_path(&repo.get_full_name())
        );

        let resp: Result<ProjectResponse, GitLabErrorResponse> = self
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

        let visibility = if core
            .config()
            .get_features()
            .has(features::CREATE_REMOTE_PRIVATE)
        {
            "private"
        } else {
            "public"
        };

        let mut body_json = json!({
            "name": repo.get_name(),
            "path": repo.get_name(),
            "visibility": visibility,
        });

        // When the repository lives under a namespace which isn't the current user's
        // personal namespace, we need to resolve the numeric namespace ID so that GitLab
        // knows where to create the new project.
        if !repo.namespace.eq_ignore_ascii_case(&current_user) {
            let namespace_id = self
                .get_namespace_id(core, service, &repo.namespace)
                .await?;
            if let Value::Object(map) = &mut body_json {
                map.insert("namespace_id".into(), json!(namespace_id));
            }
        }

        let req_body = serde_json::to_vec(&body_json).wrap_system_err(
            "Failed to serialize repository information for submission to GitLab as part of repo creation.", &[
                "Please report this issue to us by creating a new GitHub issue.",
                "Try creating your repository with `git-tool new --no-create-remote` and then pushing it to GitLab manually."
            ])?;

        let uri = format!("{}/projects", self.api_url(service));
        let resp: Result<ProjectResponse, GitLabErrorResponse> = self
            .make_request(
                core,
                service,
                Method::POST,
                &uri,
                req_body,
                vec![StatusCode::CREATED],
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
        if source
            .namespace
            .eq_ignore_ascii_case(&destination.namespace)
        {
            // A move within the same namespace is a simple rename of the project's path.
            self.rename_project(core, service, &source.get_full_name(), &destination.name)
                .await?;
        } else {
            // Moving between namespaces requires a transfer, which preserves the project's
            // path. If the caller also wants to rename the project we follow up with a
            // rename once the transfer has completed.
            let uri = format!(
                "{}/projects/{}/transfer",
                self.api_url(service),
                encode_path(&source.get_full_name())
            );

            let body = serde_json::to_vec(&json!({
                "namespace": destination.namespace,
            }))
            .wrap_system_err(
                "Failed to serialize repository information for submission to GitLab as part of repo transfer.", &[
                    "Please report this issue to us by creating a new GitHub issue.",
                    "Try transferring the repository manually on GitLab using the repository settings page."
                ])?;

            let resp: Result<ProjectResponse, GitLabErrorResponse> = self
                .make_request(core, service, Method::PUT, &uri, body, vec![StatusCode::OK])
                .await?;

            if let Err(e) = resp {
                return Err(e.into());
            }

            if !source.name.eq_ignore_ascii_case(&destination.name) {
                let transferred = format!("{}/{}", destination.namespace, source.name);
                self.rename_project(core, service, &transferred, &destination.name)
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
            "{}/projects/{}/fork",
            self.api_url(service),
            encode_path(&source.get_full_name())
        );

        let body = serde_json::to_vec(&json!({
            "namespace_path": destination.namespace,
            "name": destination.name,
            "path": destination.name,
        }))
        .wrap_system_err(
            "Failed to serialize repository information for submission to GitLab as part of repo fork.", &[
                "Please report this issue to us by creating a new GitHub issue.",
                "Try forking the repository manually on GitLab using the repository page."
            ])?;

        let resp: Result<ProjectResponse, GitLabErrorResponse> = self
            .make_request(
                core,
                service,
                Method::POST,
                &uri,
                body,
                vec![StatusCode::CREATED, StatusCode::OK],
            )
            .await?;

        match resp {
            Ok(_) => Ok(()),
            Err(e) => Err(e.into()),
        }
    }
}

impl GitLabService {
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
        let user: Result<UserProfile, GitLabErrorResponse> = self
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
            Ok(user) => Ok(user.username),
            Err(e) => Err(e.into()),
        }
    }

    async fn get_namespace_id(
        &self,
        core: &Core,
        service: &Service,
        namespace: &str,
    ) -> Result<u64, human_errors::Error> {
        let uri = format!(
            "{}/namespaces/{}",
            self.api_url(service),
            encode_path(namespace)
        );

        let resp: Result<NamespaceResponse, GitLabErrorResponse> = self
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
            Ok(ns) => Ok(ns.id),
            Err(e) => Err(e.into()),
        }
    }

    async fn rename_project(
        &self,
        core: &Core,
        service: &Service,
        full_name: &str,
        new_name: &str,
    ) -> Result<(), human_errors::Error> {
        let uri = format!(
            "{}/projects/{}",
            self.api_url(service),
            encode_path(full_name)
        );

        let body = serde_json::to_vec(&json!({
            "name": new_name,
            "path": new_name,
        }))
        .wrap_system_err(
            "Failed to serialize repository information for submission to GitLab as part of repo rename.", &[
                "Please report this issue to us by creating a new GitHub issue.",
                "Try renaming the repository manually on GitLab using the repository settings page."
            ])?;

        let resp: Result<ProjectResponse, GitLabErrorResponse> = self
            .make_request(core, service, Method::PUT, &uri, body, vec![StatusCode::OK])
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
    ) -> Result<Result<T, GitLabErrorResponse>, human_errors::Error> {
        let token = core.keychain().get_token(&service.name)?;

        let headers = vec![
            ("Accept", "application/json".to_string()),
            ("Content-Type", "application/json".to_string()),
            ("Authorization", format!("Bearer {token}")),
        ];

        let (status, bytes) = request_with_retry(core, method, uri, &headers, body).await?;

        if acceptable.contains(&status) {
            let result = serde_json::from_slice(&bytes).wrap_system_err(
                "We could not deserialize the response from GitLab because it didn't match the expected response format.",
                &["Please report this issue to us on GitHub with the trace ID for the command you were running so that we can investigate."],
            )?;

            return Ok(Ok(result));
        }

        let mut result: GitLabErrorResponse = serde_json::from_slice(&bytes).unwrap_or_default();
        result.http_status_code = status;
        Ok(Err(result))
    }
}

#[derive(Debug, Deserialize)]
struct UserProfile {
    pub username: String,
}

#[derive(Debug, Deserialize)]
struct NamespaceResponse {
    pub id: u64,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct ProjectResponse {
    pub id: u64,
}

#[derive(Debug, Default, Deserialize)]
struct GitLabErrorResponse {
    #[serde(skip)]
    pub http_status_code: StatusCode,

    #[serde(default)]
    pub message: Value,

    #[serde(default)]
    pub error: Option<String>,
}

impl GitLabErrorResponse {
    fn message_text(&self) -> String {
        if let Some(err) = &self.error {
            return err.clone();
        }

        match &self.message {
            Value::String(s) => s.clone(),
            Value::Null => String::new(),
            other => other.to_string(),
        }
    }

    /// Determines whether the error returned by GitLab indicates that the project
    /// already exists (GitLab responds with a 400 whose validation errors mention
    /// that the name or path have already been taken).
    fn already_exists(&self) -> bool {
        self.http_status_code == StatusCode::BAD_REQUEST
            && self.message_text().contains("has already been taken")
    }
}

#[allow(clippy::from_over_into)]
impl Into<Error> for GitLabErrorResponse {
    fn into(self) -> Error {
        match self.http_status_code {
            StatusCode::UNAUTHORIZED => human_errors::user(
                "You have not provided a valid authentication token for GitLab.",
                &[
                    "Generate a valid Personal Access Token with the `api` scope and add it using `git-tool auth <service>`.",
                ],
            ),
            StatusCode::FORBIDDEN => human_errors::wrap_user(
                format!("{self:?}"),
                format!(
                    "You do not have permission to perform this action on GitLab: {}",
                    self.message_text()
                ),
                &[
                    "Check that your access token has the `api` scope and that you have permission to manage this project or group.",
                ],
            ),
            StatusCode::NOT_FOUND => human_errors::wrap_user(
                format!("{self:?}"),
                "We could not find the GitLab group or project you specified.",
                &[
                    "Check that you have specified the correct group or user in the repository name and try again.",
                ],
            ),
            StatusCode::TOO_MANY_REQUESTS => human_errors::user(
                "GitLab has rate limited requests from your IP address.",
                &["Please wait until GitLab removes this rate limit before trying again."],
            ),
            status => human_errors::wrap_system(
                format!("{self:?}"),
                format!(
                    "Received an HTTP {} {} response from GitLab: {}",
                    status.as_u16(),
                    status.canonical_reason().unwrap_or_default(),
                    self.message_text()
                ),
                &[
                    "Please read the error message below and decide if there is something you can do to fix the problem, or report it to us on GitHub.",
                ],
            ),
        }
    }
}

fn encode_path(value: &str) -> String {
    // GitLab identifies projects and groups by their URL-encoded path, where the `/`
    // separators are encoded (for example `group/project` becomes `group%2Fproject`).
    // We encode every reserved character but preserve the unreserved characters
    // (`-`, `.`, `_`, `~`) so that the resulting identifiers stay readable and match
    // exactly what GitLab expects.
    const PATH_ENCODE: &AsciiSet = &NON_ALPHANUMERIC
        .remove(b'-')
        .remove(b'.')
        .remove(b'_')
        .remove(b'~');

    utf8_percent_encode(value, PATH_ENCODE).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::eq;

    fn service() -> Service {
        Service {
            name: "gl".into(),
            website: "https://gitlab.com/{{ .Repo.FullName }}".into(),
            git_url: "git@gitlab.com:{{ .Repo.FullName }}.git".into(),
            pattern: "*/*".into(),
            api: Some(ServiceAPI {
                kind: "GitLab/v4".into(),
                url: "https://gitlab.com/api/v4".into(),
            }),
        }
    }

    fn core(mocks: Vec<MockHttpRoute>) -> Core {
        Core::builder()
            .with_default_config()
            .with_mock_keychain(|mock| {
                mock.expect_get_token()
                    .with(eq("gl"))
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
                "https://gitlab.com/api/v4/user",
                200,
                r#"{ "username": "test" }"#,
            ),
            MockHttpRoute::new(
                "POST",
                "https://gitlab.com/api/v4/projects",
                201,
                r#"{ "id": 1234 }"#,
            ),
        ]);

        let repo = Repo::new("gl:test/user-repo", std::path::PathBuf::from("/"));
        GitLabService::default()
            .ensure_created(&core, &service(), &repo)
            .await
            .expect("No error should have been generated");
    }

    #[tokio::test]
    async fn test_create_namespaced_repo() {
        let core = core(vec![
            MockHttpRoute::new(
                "GET",
                "https://gitlab.com/api/v4/user",
                200,
                r#"{ "username": "test" }"#,
            ),
            MockHttpRoute::new(
                "GET",
                "https://gitlab.com/api/v4/namespaces/mygroup",
                200,
                r#"{ "id": 42 }"#,
            ),
            MockHttpRoute::new(
                "POST",
                "https://gitlab.com/api/v4/projects",
                201,
                r#"{ "id": 1234 }"#,
            ),
        ]);

        let repo = Repo::new("gl:mygroup/user-repo", std::path::PathBuf::from("/"));
        GitLabService::default()
            .ensure_created(&core, &service(), &repo)
            .await
            .expect("No error should have been generated");
    }

    #[tokio::test]
    async fn test_create_repo_exists() {
        let core = core(vec![
            MockHttpRoute::new(
                "GET",
                "https://gitlab.com/api/v4/user",
                200,
                r#"{ "username": "test" }"#,
            ),
            MockHttpRoute::new(
                "POST",
                "https://gitlab.com/api/v4/projects",
                400,
                r#"{ "message": { "name": ["has already been taken"], "path": ["has already been taken"] } }"#,
            ),
        ]);

        let repo = Repo::new("gl:test/user-repo", std::path::PathBuf::from("/"));
        GitLabService::default()
            .ensure_created(&core, &service(), &repo)
            .await
            .expect("No error should have been generated");
    }

    #[tokio::test]
    async fn test_is_created_yes() {
        let core = core(vec![MockHttpRoute::new(
            "GET",
            "https://gitlab.com/api/v4/projects/test%2Fuser-repo",
            200,
            r#"{ "id": 1234 }"#,
        )]);

        let repo = Repo::new("gl:test/user-repo", std::path::PathBuf::from("/"));
        assert!(
            GitLabService::default()
                .is_created(&core, &service(), &repo)
                .await
                .expect("No error should have been generated")
        );
    }

    #[tokio::test]
    async fn test_is_created_no() {
        let core = core(vec![MockHttpRoute::new(
            "GET",
            "https://gitlab.com/api/v4/projects/test%2Fuser-repo",
            404,
            r#"{ "message": "404 Project Not Found" }"#,
        )]);

        let repo = Repo::new("gl:test/user-repo", std::path::PathBuf::from("/"));
        assert!(
            !GitLabService::default()
                .is_created(&core, &service(), &repo)
                .await
                .expect("No error should have been generated")
        );
    }

    #[tokio::test]
    async fn test_move_same_namespace() {
        let core = core(vec![MockHttpRoute::new(
            "PUT",
            "https://gitlab.com/api/v4/projects/test%2Fuser-repo",
            200,
            r#"{ "id": 1234 }"#,
        )]);

        let src = Repo::new("gl:test/user-repo", std::path::PathBuf::from("/"));
        let dest = Repo::new("gl:test/new-name", std::path::PathBuf::from("/"));
        GitLabService::default()
            .move_repo(&core, &service(), &src, &dest)
            .await
            .expect("No error should have been generated");
    }

    #[tokio::test]
    async fn test_move_different_namespace() {
        let core = core(vec![
            MockHttpRoute::new(
                "PUT",
                "https://gitlab.com/api/v4/projects/test%2Fuser-repo/transfer",
                200,
                r#"{ "id": 1234 }"#,
            ),
            MockHttpRoute::new(
                "PUT",
                "https://gitlab.com/api/v4/projects/other%2Fuser-repo",
                200,
                r#"{ "id": 1234 }"#,
            ),
        ]);

        let src = Repo::new("gl:test/user-repo", std::path::PathBuf::from("/"));
        let dest = Repo::new("gl:other/new-name", std::path::PathBuf::from("/"));
        GitLabService::default()
            .move_repo(&core, &service(), &src, &dest)
            .await
            .expect("No error should have been generated");
    }

    #[tokio::test]
    async fn test_fork_repo() {
        let core = core(vec![MockHttpRoute::new(
            "POST",
            "https://gitlab.com/api/v4/projects/test%2Fuser-repo/fork",
            201,
            r#"{ "id": 1234 }"#,
        )]);

        let src = Repo::new("gl:test/user-repo", std::path::PathBuf::from("/"));
        let dest = Repo::new("gl:me/user-repo", std::path::PathBuf::from("/"));
        GitLabService::default()
            .fork_repo(&core, &service(), &src, &dest, true)
            .await
            .expect("No error should have been generated");
    }
}
