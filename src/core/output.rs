use super::Config;
use std::sync::Arc;
use std::io::{stdout, Write};

pub trait Output: From<Arc<Config>> + Send + Sync {
    fn writer(&self) -> Box<dyn Write + Send>;
}

pub struct StdoutOutput {
    
}

impl Output for StdoutOutput {
    fn writer(&self) -> Box<dyn Write + Send> {
        Box::new(stdout())
    }
}

impl From<Arc<Config>> for StdoutOutput {
    fn from(_: Arc<Config>) -> Self {
        Self {}
    }
}

#[cfg(test)]
pub mod mocks {
    use super::*;
    use std::{io::ErrorKind, sync::Mutex};

    pub struct MockOutput {
        written_data: Arc<Mutex<String>>,
    }

    impl ToString for MockOutput {
        fn to_string(&self) -> String {
            self.written_data.lock()
                .map(|m| m.to_string())
                .unwrap_or_default()    
        }
    }

    impl Output for MockOutput {
        fn writer(&self) -> Box<dyn Write + Send> {
            Box::new(MockOutputWriter{
                write_to: self.written_data.clone()
            })
        }
    }

    impl From<Arc<Config>> for MockOutput {
        fn from(_: Arc<Config>) -> Self {
            Self {
                written_data: Arc::new(Mutex::new(String::new()))
            }
        }
    }

    struct MockOutputWriter {
        write_to: Arc<Mutex<String>>,
    }

    impl Write for MockOutputWriter {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.write_to.lock()
                .map_err(|_| ErrorKind::Other.into())
                .map(|mut data| {
                    if let Ok(s) = String::from_utf8(buf.to_vec()) {
                        data.push_str(&s);
                        buf.len()
                    } else {
                        0
                    }
                })
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
}