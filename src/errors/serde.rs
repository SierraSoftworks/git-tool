impl From<serde_yaml::Error> for super::Error {
    fn from(err: serde_yaml::Error) -> Self {
        human_errors::wrap_user(
            err,
            "We could not decode the YAML response correctly.",
            &["Please make sure that your YAML file has been formatted correctly and then try again."],
        )
    }
}

impl From<serde_json::Error> for super::Error {
    fn from(err: serde_json::Error) -> Self {
        human_errors::wrap_user(
            err,
            "We could not decode the JSON response correctly.",
            &["Please make sure that your JSON file has been formatted correctly and then try again."],
        )
    }
}
