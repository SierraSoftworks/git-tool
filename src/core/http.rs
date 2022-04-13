use std::sync::Arc;

use reqwest::{Client, Request, Response};

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
        Ok(self.request(req).await?)
    }

    #[tracing::instrument(err, skip(req))]
    pub async fn request(&self, req: Request) -> Result<Response, Error> {
        let client = Client::new();
        Ok(client.execute(req).await?)
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
