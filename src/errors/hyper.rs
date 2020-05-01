use std::{fmt::Debug, convert};
use http::uri::InvalidUri;
use hyper::StatusCode;
use super::{user, user_with_internal, system_with_internal, Error};

impl convert::From<InvalidUri> for Error {
    fn from(err: InvalidUri) -> Self {
        user_with_internal(
            "We could not parse the URL.",
            "Please make sure that the URLs you are using are well formed and try this operation again.",
            err
        )
    }
}

impl convert::From<hyper::Error> for Error {
    fn from(err: hyper::Error) -> Self {
        if err.is_user() {
            user_with_internal(
                "We could not complete a web request to due to a redirect loop.",
                "This is likely due to a problem with the remote server, please try again later and report the problem to us on GitHub if the issue persists.", 
                err)
        } else if err.is_timeout() {
            system_with_internal(
                "We timed out making a web request.",
                "This is likely due to a problem with the remote server or your internet connection, please try again later and report the problem to us on GitHub if the issue persists.", 
                err)
        }
        else {
            system_with_internal(
                "An internal error occurred which we could not recover from.",
                "Please read the internal error below and decide if there is something you can do to fix the problem, or report it to us on GitHub.", 
                err)
        }
    }
}

impl<T> convert::From<hyper::Response<T>> for Error
    where T : Debug {
    fn from(resp: hyper::Response<T>) -> Self {
        match resp.status() {
            StatusCode::NOT_FOUND => user(
                "We received a 404 Not Found response when sending a web request.",
                "Please check that you're using the correct options and try again. If the problem persists, please open an issue with us on GitHub."),
            StatusCode::UNAUTHORIZED => user(
                "We received an 401 Unauthorized response when sending a web request.",
                "This probably means that you have not configured your access tokens correctly, please check your configuration and try again."),
            StatusCode::FORBIDDEN => user(
                "We received a 403 Forbidden response when sending a web request.",
                "This probably means that you do not have permission to access this resource, please check that you do have permission and try again."),
            _ => system_with_internal(
                format!("We received a {} status code when making a web request.", resp.status()).as_str(),
                "This is likely due to a problem with the remote server, please try again later and report the problem to us on GitHub if the issue persists.",
                resp)
        }
    }
}