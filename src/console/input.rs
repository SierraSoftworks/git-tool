use std::io::Read;
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
                        let end = *pos + n;
                        let max_read = data.len();
                        let end = if end > max_read { max_read } else { end };

                        if *pos == end {
                            return String::new();
                        }

                        let read = data[*pos..end].to_string();
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
