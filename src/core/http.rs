use std::sync::Arc;

use opentelemetry::trace::{SpanKind, StatusCode};
use reqwest::{Client, Request, Response};
use tracing::field;

use super::{Config, Error};

#[cfg(test)]
use mocktopus::macros::*;

#[cfg_attr(test, mockable)]
pub struct HttpClient {}

impl std::fmt::Debug for HttpClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "HttpClient")
    }
}

#[cfg_attr(test, mockable)]
impl HttpClient {
    #[tracing::instrument(err, skip(uri))]
    pub async fn get(&self, uri: reqwest::Url) -> Result<Response, Error> {
        let req = Request::new(reqwest::Method::GET, uri);
        self.request(req).await
    }

    #[tracing::instrument(
        err,
        skip(req),
        fields(
            otel.kind = ?SpanKind::Client,
            otel.status = ?StatusCode::Unset,
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
    pub async fn request(&self, req: Request) -> Result<Response, Error> {
        let client = Client::new();

        let response = client.execute(req).await?;

        let span_status = if response.status().is_success() {
            StatusCode::Ok
        } else {
            StatusCode::Error
        }
        .as_str();

        tracing::span::Span::current()
            .record("http.status_code", &response.status().as_u16())
            .record(
                "http.response_content_length",
                &response.content_length().unwrap_or(0),
            )
            .record("otel.status", &span_status)
            .record(
                "otel.status_message",
                &response.status().canonical_reason().unwrap_or("<none>"),
            );

        Ok(response)
    }

    #[cfg(test)]
    pub fn mock(routes: Vec<mocks::Route>) {
        mocks::mock(routes)
    }

    #[cfg(test)]
    pub fn route(method: &str, path: &str, status: u16, body: &str) -> mocks::Route {
        mocks::Route::new(method, path, status, body)
    }
}

impl From<Arc<Config>> for HttpClient {
    fn from(_: Arc<Config>) -> Self {
        Self {}
    }
}

#[cfg(test)]
mod mocks {
    use super::*;
    use mocktopus::mocking::*;

    pub struct Route {
        method: String,
        path: String,
        status: u16,
        body: String,
    }

    impl Route {
        pub fn new(method: &str, path: &str, status: u16, body: &str) -> Self {
            Self {
                method: method.into(),
                path: path.into(),
                status,
                body: body.into(),
            }
        }
    }

    pub fn mock(routes: Vec<Route>) {
        HttpClient::request.mock_safe(move |_, req| {
            for route in routes.iter() {
                if req.method().to_string() == route.method && req.url().to_string() == route.path {
                    let status = route.status;
                    let body = route.body.clone();

                    return MockResult::Return(Box::pin(async move {
                        Ok(http::Response::builder()
                            .header("Content-Type", "application/vnd.github.v3+json")
                            .status(status)
                            .body(body)
                            .unwrap()
                            .into())
                    }));
                }
            }

            panic!("Unrecognized route {} {}", req.method(), req.url());
        });
    }
}
