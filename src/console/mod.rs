#[cfg(test)]
use std::fmt::Display;
use std::io::{Read, Write, stdin, stdout};
use std::sync::Arc;

#[cfg(test)]
mod input;
#[cfg(test)]
mod output;

pub trait ConsoleProvider {
    fn input(&self) -> Box<dyn Read>;
    fn output(&self) -> Box<dyn Write + Send>;
}

pub fn default() -> Arc<dyn ConsoleProvider + Send + Sync> {
    Arc::new(DefaultConsoleProvider {})
}

#[cfg(test)]
pub fn mock() -> Arc<MockConsoleProvider> {
    Arc::new(MockConsoleProvider::new())
}

#[cfg(test)]
pub fn mock_with_input(input: &str) -> Arc<MockConsoleProvider> {
    Arc::new(MockConsoleProvider::from(input))
}

#[cfg(test)]
pub fn null() -> Arc<dyn ConsoleProvider + Send + Sync> {
    Arc::new(NullConsoleProvider {})
}

struct DefaultConsoleProvider;

impl ConsoleProvider for DefaultConsoleProvider {
    fn input(&self) -> Box<dyn Read> {
        Box::new(stdin())
    }

    fn output(&self) -> Box<dyn Write + Send> {
        Box::new(PipeSafeWriter::new(stdout()))
    }
}

/// A [`Write`] adapter which tolerates the output stream being torn down (the
/// terminal being closed, or a downstream reader in a pipe exiting) rather than
/// surfacing the resulting IO error.
///
/// Command output flows through here from every subcommand, so a broken pipe
/// would otherwise bubble up as a `human_errors::Error` (printed to the user
/// and, for system errors, reported to telemetry) or, in the case of the raw
/// print macros, panic outright. When the consumer of our output has gone away
/// there is nothing useful to do with the bytes and nowhere to report the
/// failure, so we drop the remaining output silently. This mirrors the way
/// traditional Unix tools are terminated by `SIGPIPE` without complaint, and
/// keeps `git-tool ... | head` (or closing the terminal mid-command) from
/// producing spurious errors.
///
/// Only disconnection-style errors are swallowed; genuine failures (for example
/// a full disk when output is redirected to a file) continue to propagate as
/// before.
struct PipeSafeWriter<W> {
    inner: W,
    disconnected: bool,
}

impl<W: Write> PipeSafeWriter<W> {
    fn new(inner: W) -> Self {
        Self {
            inner,
            disconnected: false,
        }
    }
}

/// Returns `true` when an IO error indicates the consumer of our output has gone
/// away, as opposed to a recoverable or reportable failure.
fn is_disconnection(err: &std::io::Error) -> bool {
    // `BrokenPipe` covers a downstream reader exiting on every platform (and is
    // what Windows maps its broken-pipe error codes to). The raw `EIO` (os error
    // 5) check covers a terminal/pty being torn down on Unix, which surfaces as
    // an uncategorised IO error rather than `BrokenPipe`.
    err.kind() == std::io::ErrorKind::BrokenPipe || err.raw_os_error() == Some(5)
}

impl<W: Write> Write for PipeSafeWriter<W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        if self.disconnected {
            return Ok(buf.len());
        }

        match self.inner.write(buf) {
            Err(e) if is_disconnection(&e) => {
                self.disconnected = true;
                Ok(buf.len())
            }
            other => other,
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        if self.disconnected {
            return Ok(());
        }

        match self.inner.flush() {
            Err(e) if is_disconnection(&e) => {
                self.disconnected = true;
                Ok(())
            }
            other => other,
        }
    }
}

#[cfg(test)]
pub struct NullConsoleProvider;

#[cfg(test)]
impl ConsoleProvider for NullConsoleProvider {
    fn input(&self) -> Box<dyn Read> {
        Box::new(std::io::empty())
    }

    fn output(&self) -> Box<dyn Write + Send> {
        Box::new(std::io::sink())
    }
}

#[cfg(test)]
pub struct MockConsoleProvider {
    input: input::MockInputReader,
    output: output::MockOutput,
}

#[cfg(test)]
impl ConsoleProvider for MockConsoleProvider {
    fn input(&self) -> Box<dyn Read> {
        Box::new(self.input.clone())
    }

    fn output(&self) -> Box<dyn Write + Send> {
        Box::new(self.output.clone())
    }
}

#[cfg(test)]
impl MockConsoleProvider {
    pub fn new() -> Self {
        Self {
            input: input::MockInputReader::from(""),
            output: output::MockOutput::default(),
        }
    }
}

#[cfg(test)]
impl From<&str> for MockConsoleProvider {
    fn from(data: &str) -> Self {
        Self {
            input: input::MockInputReader::from(data),
            output: output::MockOutput::default(),
        }
    }
}

#[cfg(test)]
impl From<String> for MockConsoleProvider {
    fn from(data: String) -> Self {
        Self {
            input: input::MockInputReader::from(data.as_str()),
            output: output::MockOutput::default(),
        }
    }
}

#[cfg(test)]
impl Display for MockConsoleProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A `Write` whose every call fails with a configurable error, used to
    /// exercise `PipeSafeWriter`'s error handling.
    struct FailingWriter {
        error: fn() -> std::io::Error,
        writes: usize,
    }

    impl FailingWriter {
        fn new(error: fn() -> std::io::Error) -> Self {
            Self { error, writes: 0 }
        }
    }

    impl Write for FailingWriter {
        fn write(&mut self, _buf: &[u8]) -> std::io::Result<usize> {
            self.writes += 1;
            Err((self.error)())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Err((self.error)())
        }
    }

    #[test]
    fn pipe_safe_writer_passes_through_successful_writes() {
        let mut buffer: Vec<u8> = Vec::new();
        let mut writer = PipeSafeWriter::new(&mut buffer);

        assert_eq!(writer.write(b"hello").unwrap(), 5);
        writer.flush().unwrap();
        assert!(!writer.disconnected);
        assert_eq!(buffer, b"hello");
    }

    #[test]
    fn pipe_safe_writer_swallows_broken_pipe() {
        let mut writer =
            PipeSafeWriter::new(FailingWriter::new(|| std::io::ErrorKind::BrokenPipe.into()));

        // The broken pipe is reported as a successful write of the whole buffer
        // rather than an error, and the writer latches into the disconnected
        // state so later writes are dropped without touching the inner writer.
        assert_eq!(writer.write(b"hello").unwrap(), 5);
        assert!(writer.disconnected);
        assert_eq!(writer.write(b"world").unwrap(), 5);
        assert_eq!(writer.inner.writes, 1);
        writer.flush().unwrap();
    }

    #[test]
    fn pipe_safe_writer_swallows_eio() {
        // `EIO` (os error 5) is what a torn-down terminal produces on Unix.
        let mut writer =
            PipeSafeWriter::new(FailingWriter::new(|| std::io::Error::from_raw_os_error(5)));

        assert_eq!(writer.write(b"hello").unwrap(), 5);
        assert!(writer.disconnected);
    }

    #[test]
    fn pipe_safe_writer_propagates_other_errors() {
        // A genuine failure (e.g. a full disk when redirected to a file) must
        // still surface rather than being silently dropped.
        let mut writer = PipeSafeWriter::new(FailingWriter::new(|| {
            std::io::ErrorKind::PermissionDenied.into()
        }));

        let err = writer.write(b"hello").unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::PermissionDenied);
        assert!(!writer.disconnected);
    }
}
