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
                        *pos = end;

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{BufRead, BufReader, Read};

    #[test]
    fn test_read_n() {
        let mut reader = MockInputReader::from("Hello, world!");
        let mut buffer = [0; 5];
        let bytes_read = reader.read(&mut buffer).unwrap();
        assert_eq!(bytes_read, 5);
        assert_eq!(&buffer[..bytes_read], b"Hello");
    }

    #[test]
    fn test_read_n_exceeding() {
        let mut reader = MockInputReader::from("Hello");
        let mut buffer = [0; 10];
        let bytes_read = reader.read(&mut buffer).unwrap();
        assert_eq!(bytes_read, 5);
        assert_eq!(&buffer[..bytes_read], b"Hello");
    }

    #[test]
    fn test_read_n_empty() {
        let mut reader = MockInputReader::from("");
        let mut buffer = [0; 5];
        let bytes_read = reader.read(&mut buffer).unwrap();
        assert_eq!(bytes_read, 0);
        assert_eq!(&buffer[..bytes_read], b"");
    }

    #[test]
    fn test_read_into_bufreader() {
        let data = "Hello, world!";
        let reader = MockInputReader::from(data);
        let mut bufreader = BufReader::new(reader);
        let mut output = String::new();
        bufreader.read_to_string(&mut output).unwrap();
        assert_eq!(output, data);
    }

    #[test]
    fn test_read_line_with_bufreader() {
        let data = "Hello\nworld!";
        let reader = MockInputReader::from(data);
        let mut bufreader = BufReader::new(reader);
        let mut output = String::new();
        bufreader.read_line(&mut output).unwrap();
        assert_eq!(output, "Hello\n");
    }
}
