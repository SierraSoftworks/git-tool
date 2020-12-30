//! Contains various utility types and macros useful for testing hyper clients.
//!
//! # Macros
//! The `mock_connector!` and `mock_connector_in_order!` macros can be used to
//! feed a client with preset data. That way you can control exactly what it will
//! see, confining the test-case to its own sandbox that way.
//!
//! All types they define are public to allow them to be used in other unit-tests.
//! Please note that integration tests cannot share their mock types anyway, as each
//! integration test goes into its own binary.
//!
//! # Usage
//!
//! Set it up for use in tests in `Cargo.toml`
//!
//! ```toml
//! [dev-dependencies]
//! yup-hyper-mock = "*"
//! log = "*"  # log macros are used within yup-hyper-mock
//! ```
//!
//! Link it into your `src/(lib.rs|main.rs)`
//!
//! ```Rust
//! #[cfg(test)] #[macro_use]
//! extern crate "yup-hyper-mock" as hyper_mock
//! ```

#[macro_use]
extern crate log;

use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use futures;
use futures::lock::Mutex;
use futures::Future;
use hyper::{service::Service, Uri};

mod streams;
pub use crate::streams::MockPollStream;

/// This macro maps host URLs to a respective reply, which is given in plain-text.
/// It ignores everything that is written to it.
#[macro_export]
macro_rules! mock_connector (
    ($name:ident {
        $($url:expr => $res:expr)*
    }) => (
        #[derive(Clone)]
        pub struct $name($crate::HostToReplyConnector);

        impl Default for $name {
            fn default() -> Self {
                let mut c = Self(Default::default());
                $(c.0.m.insert($url.to_string(), $res.to_string());)*
                c
            }
        }

        impl hyper::service::Service<hyper::Uri> for $name {
            type Response = $crate::MockPollStream;
            type Error = std::io::Error;
            type Future = std::pin::Pin<Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>> + Send>>;

            fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
                self.0.poll_ready(cx)
            }

            fn call(&mut self, req: hyper::Uri) -> Self::Future {
                self.0.call(req)
            }
        }
    )
);

/// A `Connect` which provides a single reply stream per host.
///
/// The mapping is done from full host url (e.g. `http://host.name.com`) to the
/// singular reply the host is supposed to make.
#[derive(Default, Clone)]
pub struct HostToReplyConnector {
    pub m: HashMap<String, String>,
}

impl Service<Uri> for HostToReplyConnector {
    type Response = crate::MockPollStream;
    type Error = std::io::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Uri) -> Self::Future {
        debug!("HostToReplyConnector::connect({:?})", req);

        match (|| {
            // ignore port for now
            self.m.get(&format!("{}://{}", req.scheme()?, req.host()?))
        })() {
            Some(res) => Box::pin(futures::future::ok(MockPollStream::new(
                res.clone().into_bytes(),
            ))),
            None => panic!("HostToReplyConnector doesn't know url {}", req),
        }
    }
}

/// This macro yields all given server replies in the order they are given.
/// The destination host URL doesn't matter at all.
#[macro_export]
macro_rules! mock_connector_in_order (
    ($name:ident {
        $( $res:expr )*
    }) => (
        #[derive(Clone)]
        pub struct $name($crate::SequentialConnector);

        impl Default for $name {
            fn default() -> $name {
                Self($crate::SequentialConnector::new(vec![
                    $($res.to_string(),)*
                ]))
            }
        }

        impl hyper::service::Service<hyper::Uri> for $name {
            type Response = $crate::MockPollStream;
            type Error = std::io::Error;
            type Future = std::pin::Pin<Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>> + Send>>;

            fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
                self.0.poll_ready(cx)
            }

            fn call(&mut self, req: hyper::Uri) -> Self::Future {
                self.0.call(req)
            }
        }
    )
);

/// A connector which requires you to implement the `Default` trait, allowing you
/// to determine the data it should be initialized with
#[derive(Clone)]
pub struct SequentialConnector {
    pub content: Arc<[String]>,
    pub current: Arc<Mutex<usize>>,
}

impl SequentialConnector {
    pub fn new(content: impl Into<Box<[String]>>) -> Self {
        let content = content.into();
        assert!(
            content.len() != 0,
            "Not a single streamer return value specified"
        );

        SequentialConnector {
            content: content.into(),
            current: Arc::new(Mutex::new(0)),
        }
    }
}

impl Service<Uri> for SequentialConnector {
    type Response = crate::MockPollStream;
    type Error = std::io::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Uri) -> Self::Future {
        debug!("SequentialConnector::connect({:?})", req);

        let content = self.content.clone();
        let current = self.current.clone();
        Box::pin(async move {
            let mut current = current.lock().await;
            let data = content[*current].clone().into_bytes();
            *current = *current + 1;
            Ok(MockPollStream::new(data))
        })
    }
}
