use std::io::{stdin, stdout, Read, Write};
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
        Box::new(stdout())
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
impl ToString for MockConsoleProvider {
    fn to_string(&self) -> String {
        self.output.to_string()
    }
}
