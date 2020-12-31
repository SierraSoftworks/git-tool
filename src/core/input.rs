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
    use std::sync::RwLock;

    pub struct MockInput {
        readable_data: Arc<RwLock<String>>,
        position: Arc<RwLock<usize>>,
    }

    impl Input for MockInput {
        fn reader(&self) -> Box<dyn Read + Send> {
            Box::new(MockInputReader {
                readable_data: self.readable_data.clone(),
                position: self.position.clone(),
            })
        }
    }

    impl MockInput {
        pub fn set_data(&mut self, data: &str) {
            self.readable_data
                .write()
                .map(|mut readable_data| readable_data.push_str(data))
                .unwrap();

            self.position.write().map(|mut pos| *pos = 0).unwrap();
        }
    }

    impl From<Arc<Config>> for MockInput {
        fn from(_: Arc<Config>) -> Self {
            Self {
                readable_data: Arc::new(RwLock::new(String::new())),
                position: Arc::new(RwLock::new(0)),
            }
        }
    }

    struct MockInputReader {
        readable_data: Arc<RwLock<String>>,
        position: Arc<RwLock<usize>>,
    }

    impl MockInputReader {
        fn read_n(&mut self, n: usize) -> std::io::Result<String> {
            self.readable_data
                .read()
                .map(|data| {
                    self.position
                        .write()
                        .and_then(|mut pos| {
                            let max_read = data.len() - *pos;
                            let n = if n > max_read { max_read } else { n };

                            if n == 0 {
                                return Ok(String::new());
                            }

                            let read = data[*pos..n].to_string();
                            *pos += n;

                            Ok(read)
                        })
                        .unwrap()
                })
                .map_err(|_| std::io::ErrorKind::Other.into())
        }
    }

    impl Read for MockInputReader {
        fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
            let data = self.read_n(buf.len())?;
            buf[..data.len()].copy_from_slice(data.as_bytes());
            Ok(data.len())
        }
    }
}
