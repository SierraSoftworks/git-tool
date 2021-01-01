use std::{
    io::{BufRead, BufReader, Write},
    sync::{Arc, Mutex},
};

use crate::{
    core::{Input, Output},
    errors,
};

use super::Error;

pub struct Prompter {
    writer: Arc<Mutex<dyn Write + Send>>,
    reader: Arc<Mutex<dyn BufRead + Send>>,
}

impl Prompter {
    pub fn new<I, O>(input: &I, output: &O) -> Self
    where
        I: Input,
        O: Output,
    {
        Self {
            writer: Arc::new(Mutex::new(output.writer())),
            reader: Arc::new(Mutex::new(BufReader::new(input.reader()))),
        }
    }

    pub fn prompt<V>(&mut self, message: &str, validate: V) -> Result<Option<String>, Error>
    where
        V: Fn(&str) -> bool,
    {
        let mut line = String::default();

        for _i in 0..3 {
            let mut writer = self
                .writer
                .lock()
                .map_err(|_| errors::system(
                    "We could not acquire a synchronization lock on the terminal stdout stream when attempting to prompt you for input.", 
                    "Please try again and if this does not resolve the problem, create a GitHub issue explaining how to reproduce the issue so that we can investigate further."))?;

            write!(writer, "{}", message)?;
            writer.flush()?;

            let mut reader = self.reader.lock().map_err(|_| errors::system(
                "We could not acquire a synchronization lock on the terminal's stdin stream when attempting to prompt you for input.", 
                "Please try again and if this does not resolve the problem, create a GitHub issue explaining how to reproduce the issue so that we can investigate further."))?;

            let n = reader.read_line(&mut line)?;
            if n == 0 {
                return Ok(None);
            }

            if !validate(&line.trim()) {
                line.clear()
            } else {
                return Ok(Some(line.trim().into()));
            }
        }

        Ok(None)
    }

    pub fn prompt_bool(&mut self, message: &str) -> Result<Option<bool>, Error> {
        if let Some(answer) = self.prompt(message, |l| {
            l.to_lowercase() == "y" || l.to_lowercase() == "n"
        })? {
            Ok(Some(answer.to_lowercase() == "y"))
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::core::{input::mocks::MockInput, output::mocks::MockOutput, Config};

    #[test]
    fn prompt_for_any() {
        let config = Arc::new(Config::default());

        let mut input = MockInput::from(config.clone());
        let output = MockOutput::from(config.clone());

        input.set_data("123\n");

        let mut prompter = Prompter::new(&input, &output);

        assert_eq!(
            prompter
                .prompt("Enter a number: ", |l| {
                    let n: Option<u32> = l.parse().ok();
                    n.is_some()
                })
                .unwrap(),
            Some("123".into()),
        );

        assert_eq!(output.to_string(), "Enter a number: ");
    }

    #[test]
    fn prompt_eof() {
        let config = Arc::new(Config::default());

        let input = MockInput::from(config.clone());
        let output = MockOutput::from(config.clone());
        let mut prompter = Prompter::new(&input, &output);

        assert_eq!(
            prompter
                .prompt("Enter a number: ", |l| {
                    let n: Option<u32> = l.parse().ok();
                    n.is_some()
                })
                .unwrap(),
            None,
        );

        assert_eq!(output.to_string(), "Enter a number: ");
    }

    #[test]
    fn prompt_retry() {
        let config = Arc::new(Config::default());

        let mut input = MockInput::from(config.clone());
        let output = MockOutput::from(config.clone());
        input.set_data("\nnan\n123\n");

        let mut prompter = Prompter::new(&input, &output);

        assert_eq!(
            prompter
                .prompt("Enter a number: ", |l| {
                    let n: Option<u32> = l.parse().ok();
                    n.is_some()
                })
                .unwrap(),
            Some("123".into()),
        );

        assert_eq!(
            output.to_string(),
            "Enter a number: Enter a number: Enter a number: "
        );
    }

    #[test]
    fn prompt_multiple() {
        let config = Arc::new(Config::default());

        let mut input = MockInput::from(config.clone());
        let output = MockOutput::from(config.clone());
        input.set_data("a\nb\n");

        let mut prompter = Prompter::new(&input, &output);

        assert_eq!(
            prompter.prompt("First prompt: ", |_| true).unwrap(),
            Some("a".into()),
        );

        assert_eq!(
            prompter.prompt("Second prompt: ", |_| true).unwrap(),
            Some("b".into()),
        );

        assert_eq!(output.to_string(), "First prompt: Second prompt: ");
    }
}
