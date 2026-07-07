use std::sync::Arc;

use reqwest::{Client, Request, Response};
use tracing_batteries::prelude::*;

use crate::errors::HumanErrorResultExt;

use super::Config;

#[cfg(test)]
use mockall::automock;

#[cfg_attr(test, automock)]
#[async_trait::async_trait]
pub trait HttpClient: Send + Sync {
    #[tracing::instrument(err, skip(self, uri))]
    async fn get(&self, uri: reqwest::Url) -> Result<Response, human_errors::Error> {
        let req = Request::new(reqwest::Method::GET, uri);
        self.request(req).await
    }

    async fn request(&self, req: Request) -> Result<Response, human_errors::Error>;

    /// The underlying [`reqwest::Client`], for code that needs a real client
    /// rather than this trait — e.g. the self-updater, which hands a
    /// `reqwest::Client` to `update-rs`. It shares this client's configuration
    /// and connection pool.
    fn reqwest_client(&self) -> Client;
}

pub fn client() -> Arc<dyn HttpClient + Send + Sync> {
    Arc::new(TrueHttpClient {
        client: build_client(),
    })
}

/// Build Git-Tool's shared reqwest client, carrying a default `User-Agent` so
/// requests that don't set their own are still attributed to Git-Tool (GitHub
/// requires one). Call sites that set a per-request `User-Agent` still override
/// it.
fn build_client() -> Client {
    Client::builder()
        .user_agent(version!("Git-Tool/"))
        .build()
        .unwrap_or_else(|_| Client::new())
}

struct TrueHttpClient {
    client: Client,
}

/// Wraps an [`HttpClient`] so that every request records a privacy-preserving
/// telemetry event describing the request's method and response status — never
/// its URL, which may identify the service (and repository) being accessed.
///
/// Requests made through [`HttpClient::reqwest_client`] bypass this wrapper; the
/// call sites which need a raw client (such as the self-updater) record their own
/// higher-level events instead.
pub(super) struct InstrumentedHttpClient {
    pub(super) inner: Arc<dyn HttpClient + Send + Sync>,
    pub(super) analytics: super::Analytics,
}

#[async_trait::async_trait]
impl HttpClient for InstrumentedHttpClient {
    async fn request(&self, req: Request) -> Result<Response, human_errors::Error> {
        let method = req.method().to_string();

        let result = self.inner.request(req).await;

        let mut properties = vec![
            ("method", method),
            (
                "status",
                if result.is_ok() {
                    "succeeded".to_string()
                } else {
                    "failed".to_string()
                },
            ),
        ];
        if let Ok(response) = &result {
            properties.push(("code", response.status().as_u16().to_string()));
        }

        self.analytics.record_event("http::request", properties);

        result
    }

    fn reqwest_client(&self) -> Client {
        self.inner.reqwest_client()
    }
}

impl std::fmt::Debug for dyn HttpClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "HttpClient")
    }
}

#[cfg(test)]
pub fn mock(routes: Vec<mocks::MockHttpRoute>) -> Arc<dyn HttpClient + Send + Sync> {
    mocks::mock_http_client(routes)
}

#[async_trait::async_trait]
impl HttpClient for TrueHttpClient {
    #[tracing::instrument(
        err,
        skip(self, req),
        fields(
            otel.kind = ?OpenTelemetrySpanKind::Client,
            otel.status_code = 0,
            otel.status_message = EmptyField,
            http.method = %req.method(),
            http.url = %req.url(),
            http.target = req.url().path(),
            http.host = req.url().host_str().unwrap_or("<none>"),
            http.scheme = req.url().scheme(),
            http.status_code = EmptyField,
            http.response_content_length = EmptyField,
        )
    )]
    async fn request(&self, req: Request) -> Result<Response, human_errors::Error> {
        let response = self.client.execute(req).await.to_human_error()?;

        if !response.status().is_success() {
            Span::current().record("otel.status_code", 2_u32).record(
                "otel.status_message",
                response.status().canonical_reason().unwrap_or("<none>"),
            );
        }

        Span::current()
            .record("http.status_code", response.status().as_u16())
            .record(
                "http.response_content_length",
                response.content_length().unwrap_or(0),
            );

        Ok(response)
    }

    fn reqwest_client(&self) -> Client {
        self.client.clone()
    }
}

impl From<Arc<Config>> for TrueHttpClient {
    fn from(_: Arc<Config>) -> Self {
        Self {
            client: build_client(),
        }
    }
}

#[cfg(test)]
pub(super) mod mocks {
    use super::*;
    use http::response;
    use mockall::predicate::*;

    pub struct MockHttpRoute {
        method: String,
        path: String,
        status: u16,
        body: String,
    }

    impl MockHttpRoute {
        pub fn new(method: &str, path: &str, status: u16, body: &str) -> Self {
            Self {
                method: method.into(),
                path: path.into(),
                status,
                body: body.into(),
            }
        }
    }

    pub fn mock_http_client(routes: Vec<MockHttpRoute>) -> Arc<dyn HttpClient + Send + Sync> {
        let mut mock = MockHttpClient::new();

        for route in routes {
            let method = route.method;
            let path = route.path;
            let body = route.body;
            let status = route.status;

            mock.expect_request()
                .withf(move |req| {
                    req.method().as_str().eq_ignore_ascii_case(&method)
                        && (req.url().path().eq_ignore_ascii_case(&path)
                            || req.url().as_str().eq_ignore_ascii_case(&path))
                })
                .returning(move |_| {
                    let res = response::Builder::new()
                        .header("Content-Type", "application/vnd.github.v3+json")
                        .status(status)
                        .body(body.clone())
                        .unwrap()
                        .into();
                    Ok(res)
                });
        }

        Arc::new(mock)
    }
}
