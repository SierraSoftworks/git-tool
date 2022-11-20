use std::sync::Arc;

use opentelemetry::trace::SpanKind;
use reqwest::{Client, Request, Response};
use tracing::field;

use super::{Config, Error};

#[cfg(test)]
use mockall::automock;

#[cfg_attr(test, automock)]
#[async_trait::async_trait]
pub trait HttpClient: Send + Sync {
    #[tracing::instrument(err, skip(self, uri))]
    async fn get(&self, uri: reqwest::Url) -> Result<Response, Error> {
        let req = Request::new(reqwest::Method::GET, uri);
        self.request(req).await
    }

    async fn request(&self, req: Request) -> Result<Response, Error>;
}

pub fn client() -> Arc<dyn HttpClient + Send + Sync> {
    Arc::new(TrueHttpClient {})
}

struct TrueHttpClient {}

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
            otel.kind = ?SpanKind::Client,
            otel.status_code = 0,
            otel.status_message = field::Empty,
            http.method = %req.method(),
            http.url = %req.url(),
            http.target = req.url().path(),
            http.host = req.url().host_str().unwrap_or("<none>"),
            http.scheme = req.url().scheme(),
            http.status_code = field::Empty,
            http.response_content_length = field::Empty,
        )
    )]
    async fn request(&self, req: Request) -> Result<Response, Error> {
        let client = Client::new();

        let response = client.execute(req).await?;

        if !response.status().is_success() {
            tracing::span::Span::current()
                .record("otel.status_code", &2_u32)
                .record(
                    "otel.status_message",
                    &response.status().canonical_reason().unwrap_or("<none>"),
                );
        }

        tracing::span::Span::current()
            .record("http.status_code", &response.status().as_u16())
            .record(
                "http.response_content_length",
                &response.content_length().unwrap_or(0),
            );

        Ok(response)
    }
}

impl From<Arc<Config>> for TrueHttpClient {
    fn from(_: Arc<Config>) -> Self {
        Self {}
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
