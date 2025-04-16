use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub const CREATE_REMOTE: &str = "create_remote";
pub const CREATE_REMOTE_PRIVATE: &str = "create_remote_private";
pub const CHECK_EXISTS: &str = "check_exists";
pub const MOVE_REMOTE: &str = "move_remote";
pub const FORK_REMOTE: &str = "fork_remote";

pub const OPEN_NEW_REPO: &str = "open_new_repo_in_default_app";
pub const ALWAYS_OPEN_BEST_MATCH: &str = "always_open_best_match";

pub const TELEMETRY: &str = "telemetry";
pub const CHECK_FOR_UPDATES: &str = "check_for_updates";

lazy_static! {
    pub static ref ALL: Vec<&'static str> = vec![
        CREATE_REMOTE,
        CREATE_REMOTE_PRIVATE,
        CHECK_EXISTS,
        MOVE_REMOTE,
        FORK_REMOTE,
        OPEN_NEW_REPO,
        ALWAYS_OPEN_BEST_MATCH,
        TELEMETRY,
        CHECK_FOR_UPDATES,
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
        self.flags.get(flag).copied().unwrap_or_default()
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
            .with(MOVE_REMOTE, true)
            .with(FORK_REMOTE, true)
            .with(TELEMETRY, false)
            .with(CHECK_FOR_UPDATES, true)
            .with(CHECK_EXISTS, true)
    }

    pub fn with_features(self, features: &Features) -> Self {
        let mut flags = self.flags;
        for (key, &val) in features.flags.iter() {
            flags.insert(key.clone(), val);
        }

        Self { flags }
    }

    pub fn build(self) -> Features {
        Features { flags: self.flags }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default() {
        assert!(Features::default().has(CREATE_REMOTE));
        assert!(Features::default().has(CHECK_FOR_UPDATES));
    }
}
