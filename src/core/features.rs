use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Copy, Clone)]
pub struct Features {
    native_clone: bool,
    create_remote: bool,
    http_transport: bool,
}

impl Features {
    pub fn builder() -> FeaturesBuilder {
        FeaturesBuilder {
            create_remote: true,
            http_transport: false
        }
    }

    pub fn use_http_transport(&self) -> bool {
        self.http_transport
    }

    pub fn should_create_remote(&self) -> bool {
        self.create_remote
    }
}

pub struct FeaturesBuilder {
    create_remote: bool,
    http_transport: bool
}

impl FeaturesBuilder {
    pub fn with_create_remote(self, enabled: bool) -> Self {
        Self {
            create_remote: enabled,
            http_transport: self.http_transport
        }
    }

    pub fn with_use_http_transport(self, enabled: bool) -> Self {
        Self {
            create_remote: self.create_remote,
            http_transport: enabled
        }
    }

    pub fn build(self) -> Features {
        Features {
            create_remote: self.create_remote,
            http_transport: self.http_transport,
            native_clone: true
        }
    }
}

impl Default for Features {
    fn default() -> Self {
        Self {
            native_clone: false,
            create_remote: true,
            http_transport: false,
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