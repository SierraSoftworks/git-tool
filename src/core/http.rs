use std::sync::Arc;

use http::{Request, Response, Uri};
use hyper::client::HttpConnector;
use hyper::{Body, Client};

use super::{errors, Config, Error};

#[cfg(test)]
use mocktopus::macros::*;

#[cfg_attr(test, mockable)]
pub struct HttpClient {}

#[cfg_attr(test, mockable)]
impl HttpClient {
    pub async fn get(&self, uri: Uri) -> Result<Response<Body>, Error> {
        let req = Request::builder()
            .method("GET")
            .uri(uri.clone())
            .body(Body::empty())
            .map_err(|err| errors::system_with_internal(
                &format!("Unable to construct web request to '{}' due to an internal error.", uri),
                "Please report this issue to us on GitHub so that we can work with you to resolve it.",
                err))?;

        Ok(self.request(req).await?)
    }

    pub async fn request(&self, req: Request<Body>) -> Result<Response<Body>, Error> {
        let client = Client::builder().build(hyper_tls::HttpsConnector::<HttpConnector>::new());
        Ok(client.request(req).await?)
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
                if req.method().to_string() == route.method && req.uri().to_string() == route.path {
                    let status = route.status;
                    let body = route.body.clone();

                    return MockResult::Return(Box::pin(async move {
                        Ok(Response::builder()
                            .header("Content-Type", "application/vnd.github.v3+json")
                            .status(status)
                            .body(Body::from(body))
                            .unwrap())
                    }));
                }
            }

            panic!("Unrecognized route {} {}", req.method(), req.uri());
        });
    }
}
