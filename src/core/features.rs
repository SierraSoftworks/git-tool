use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Copy, Clone)]
pub struct Features {
    native_clone: bool,
    create_remote: bool,
    http_transport: bool,
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