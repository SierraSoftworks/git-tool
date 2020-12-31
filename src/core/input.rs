use super::Config;
use std::io::{stdin, Read};
use std::sync::Arc;

pub trait Input: From<Arc<Config>> + Send + Sync {
    fn reader(&self) -> Box<dyn Read + Send>;
}

pub struct StdinInput {}

impl Input for StdinInput {
    fn reader(&self) -> Box<dyn Read + Send> {
        Box::new(stdin())
    }
}

impl From<Arc<Config>> for StdinInput {
    fn from(_: Arc<Config>) -> Self {
        Self {}
    }
}

#[cfg(test)]
pub mod mocks {
    use super::*;
    use std::{io::ErrorKind, sync::Mutex};

    pub struct MockInput {
        readable_data: Arc<Mutex<String>>,
    }

    impl ToString for MockInput {
        fn to_string(&self) -> String {
            self.readable_data
                .lock()
                .map(|m| m.to_string())
                .unwrap_or_default()
        }
    }

    impl Input for MockInput {
        fn reader(&self) -> Box<dyn Read + Send> {
            Box::new(MockOutputReader {
                read_from: self.readable_data.clone(),
                position: 0,
            })
        }
    }

    impl MockInput {
        pub fn set_data(&mut self, data: &str) {
            self.readable_data
                .lock()
                .map(|mut readable_data| readable_data.push_str(data))
                .unwrap();
        }
    }

    impl From<Arc<Config>> for MockInput {
        fn from(_: Arc<Config>) -> Self {
            Self {
                readable_data: Arc::new(Mutex::new(String::new())),
            }
        }
    }

    struct MockOutputReader {
        read_from: Arc<Mutex<String>>,
        position: usize,
    }

    impl Read for MockOutputReader {
        fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
            self.read_from
                .lock()
                .and_then(|data| {
                    let mut n = buf.len();
                    if data.len() - self.position < n {
                        n = data.len() - self.position;
                    }

                    buf[..n].copy_from_slice(&data.as_bytes()[self.position..n]);
                    Ok(n)
                })
                .map_err(|_| std::io::ErrorKind::Other.into())
        }
    }
}
