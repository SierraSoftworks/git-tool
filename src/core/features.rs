use serde::{Deserialize, Serialize};

fn default_as_true() -> bool {
    true
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone)]
pub struct Features {
    #[serde(default = "default_as_true")]
    native_clone: bool,
    #[serde(default = "default_as_true")]
    create_remote: bool,
    #[serde(default)]
    http_transport: bool,
    #[serde(default = "default_as_true")]
    create_remote_private: bool,
}

impl Default for Features {
    fn default() -> Self {
        Self {
            native_clone: false,
            create_remote: true,
            http_transport: false,
            create_remote_private: true,
        }
    }
}

impl Features {
    #[cfg(test)]
    pub fn builder() -> FeaturesBuilder {
        FeaturesBuilder {
            create_remote: true,
            create_remote_private: true,
            http_transport: false,
        }
    }

    pub fn use_http_transport(&self) -> bool {
        self.http_transport
    }

    pub fn create_remote(&self) -> bool {
        self.create_remote
    }

    pub fn create_remote_private(&self) -> bool {
        self.create_remote_private
    }
}

#[cfg(test)]
pub struct FeaturesBuilder {
    create_remote: bool,
    create_remote_private: bool,
    http_transport: bool,
}

#[cfg(test)]
impl FeaturesBuilder {
    pub fn with_create_remote(self, enabled: bool) -> Self {
        Self {
            create_remote: enabled,
            create_remote_private: self.create_remote_private,
            http_transport: self.http_transport,
        }
    }

    pub fn with_use_http_transport(self, enabled: bool) -> Self {
        Self {
            create_remote: self.create_remote,
            create_remote_private: self.create_remote_private,
            http_transport: enabled,
        }
    }

    pub fn build(self) -> Features {
        Features {
            create_remote: self.create_remote,
            http_transport: self.http_transport,
            native_clone: true,
            create_remote_private: self.create_remote_private,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Features;

    #[test]
    fn default() {
        assert_eq!(Features::default().native_clone, false);
        assert_eq!(Features::default().create_remote, true);
        assert_eq!(Features::default().http_transport, false);
    }
}
