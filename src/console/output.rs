use std::fmt::Display;
use std::io::Write;
use std::{
    io::ErrorKind,
    sync::{Arc, Mutex},
};

#[derive(Clone)]
pub struct MockOutput {
    written_data: Arc<Mutex<String>>,
}

impl MockOutput {
    #[allow(dead_code)]
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

impl Display for MockOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.written_data
            .lock()
            .map(|m| write!(f, "{}", m.as_str()))
            .unwrap_or_else(|_| write!(f, ""))
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
