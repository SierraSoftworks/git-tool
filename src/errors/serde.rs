impl super::HumanErrorExt for serde_yaml::Error {
    fn to_human_error(self) -> human_errors::Error {
        human_errors::wrap_user(
            self,
            "We could not decode the YAML response correctly.",
            &[
                "Please make sure that your YAML file has been formatted correctly and then try again.",
            ],
        )
    }
}

impl super::HumanErrorExt for serde_json::Error {
    fn to_human_error(self) -> human_errors::Error {
        human_errors::wrap_user(
            self,
            "We could not decode the JSON response correctly.",
            &[
                "Please make sure that your JSON file has been formatted correctly and then try again.",
            ],
        )
    }
}
