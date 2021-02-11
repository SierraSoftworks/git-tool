use std::io::{stdout, Write};

#[cfg(test)]
use mocktopus::{macros::*, mocking::*};

#[cfg_attr(test, mockable)]
pub fn output() -> Box<dyn Write + Send> {
    Box::new(stdout())
}

#[cfg(test)]
pub fn mock() -> mocks::MockOutput {
    let writer = mocks::MockOutput::default();
    let sacrificial_writer = writer.clone();
    output.mock_safe(move || MockResult::Return(Box::new(sacrificial_writer.clone())));
    writer
}

#[cfg(test)]
#[allow(dead_code)]
pub mod mocks {
    use super::*;
    use std::{
        io::ErrorKind,
        sync::{Arc, Mutex},
    };

    #[derive(Clone)]
    pub struct MockOutput {
        written_data: Arc<Mutex<String>>,
    }

    impl MockOutput {
        pub fn clear(&self) {
            self.written_data
                .lock()
                .map(|mut m| m.clear())
                .unwrap_or_default();
        }
    }

    impl Default for MockOutput {
        fn default() -> Self {
            Self {
                written_data: Arc::new(Mutex::new(String::new())),
            }
        }
    }

    impl ToString for MockOutput {
        fn to_string(&self) -> String {
            self.written_data
                .lock()
                .map(|m| m.to_string())
                .unwrap_or_default()
        }
    }

    impl Write for MockOutput {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.written_data
                .lock()
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
