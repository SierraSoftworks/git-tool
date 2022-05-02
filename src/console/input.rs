use std::io::{stdin, Read};

#[cfg(test)]
use mocktopus::{macros::*, mocking::*};

#[cfg_attr(test, mockable)]
pub fn input() -> Box<dyn Read + Send> {
    Box::new(stdin())
}

#[cfg(test)]
pub fn mock(data: &str) {
    let reader = mocks::MockInputReader::from(data);
    input.mock_safe(move || MockResult::Return(Box::new(reader.clone())))
}

#[cfg(test)]
pub mod mocks {
    use super::*;
    use std::sync::{Arc, RwLock};

    #[derive(Clone)]
    pub struct MockInputReader {
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
                        .map(|mut pos| {
                            let max_read = data.len() - *pos;
                            let n = if n > max_read { max_read } else { n };

                            if n == 0 {
                                return String::new();
                            }

                            let read = data[*pos..n].to_string();
                            *pos += n;

                            read
                        })
                        .unwrap()
                })
                .map_err(|_| std::io::ErrorKind::Other.into())
        }
    }

    impl From<&str> for MockInputReader {
        fn from(data: &str) -> Self {
            Self {
                readable_data: Arc::new(RwLock::new(data.into())),
                position: Arc::new(RwLock::new(0)),
            }
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
