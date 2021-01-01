use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub const HTTP_TRANSPORT: &str = "http_transport";

pub const CREATE_REMOTE: &str = "create_remote";
pub const CREATE_REMOTE_PRIVATE: &str = "create_remote_private";

pub const OPEN_NEW_REPO: &str = "open_new_repo_in_default_app";

pub const TELEMETRY: &str = "telemetry";

lazy_static! {
    pub static ref ALL: Vec<&'static str> = vec![
        HTTP_TRANSPORT,
        CREATE_REMOTE,
        CREATE_REMOTE_PRIVATE,
        OPEN_NEW_REPO,
        TELEMETRY
    ];
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Features {
    #[serde(flatten)]
    flags: HashMap<String, bool>,
}

impl Default for Features {
    fn default() -> Self {
        Self::builder().build()
    }
}

impl Features {
    pub fn builder() -> FeaturesBuilder {
        FeaturesBuilder {
            flags: HashMap::new(),
        }
        .with_defaults()
    }

    pub fn has(&self, flag: &str) -> bool {
        self.flags.get(flag).map(|&v| v).unwrap_or_default()
    }

    pub fn to_builder(&self) -> FeaturesBuilder {
        FeaturesBuilder {
            flags: self.flags.clone(),
        }
    }
}

pub struct FeaturesBuilder {
    flags: HashMap<String, bool>,
}

impl FeaturesBuilder {
    pub fn with(self, flag: &str, enabled: bool) -> Self {
        let mut flags = self.flags;
        flags.insert(flag.into(), enabled);
        Self { flags }
    }

    pub fn with_defaults(self) -> Self {
        self.with(CREATE_REMOTE, true)
            .with(CREATE_REMOTE_PRIVATE, true)
            .with(TELEMETRY, true)
    }

    pub fn build(self) -> Features {
        Features { flags: self.flags }
    }
}

#[cfg(test)]
mod tests {
    use super::{Features, CREATE_REMOTE, HTTP_TRANSPORT};

    #[test]
    fn default() {
        assert_eq!(Features::default().has(CREATE_REMOTE), true);
        assert_eq!(Features::default().has(HTTP_TRANSPORT), false);
    }
}
