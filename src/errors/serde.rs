use super::{user_with_internal, Error};

impl std::convert::From<serde_yaml::Error> for Error {
    fn from(err: serde_yaml::Error) -> Self {
        user_with_internal(
            "We could not decode the YAML response correctly.",
            "Please make sure that your YAML file has been formatted correctly and then try again.",
            err,
        )
    }
}

impl std::convert::From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        user_with_internal(
            "We could not decode the JSON response correctly.",
            "Please make sure that your JSON file has been formatted correctly and then try again.",
            err,
        )
    }
}
