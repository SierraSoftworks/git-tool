use std::{
    io::{BufRead, BufReader, Write},
    sync::{Arc, Mutex},
};

use crate::{console, errors};

use super::Error;

pub struct Prompter {
    writer: Arc<Mutex<dyn Write + Send>>,
    reader: Arc<Mutex<dyn BufRead + Send>>,
}

impl Prompter {
    pub fn new() -> Self {
        Self {
            writer: Arc::new(Mutex::new(console::output::output())),
            reader: Arc::new(Mutex::new(BufReader::new(console::input::input()))),
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

    pub fn prompt_bool(
        &mut self,
        message: &str,
        default: Option<bool>,
    ) -> Result<Option<bool>, Error> {
        if let Some(answer) = self.prompt(message, |l| {
            l.to_lowercase() == "y" || l.to_lowercase() == "n"
        })? {
            Ok(Some(answer.to_lowercase() == "y"))
        } else {
            Ok(default)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prompt_for_any() {
        console::input::mock("123\n");
        let output = console::output::mock();

        let mut prompter = Prompter::new();

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
        console::input::mock("");
        let output = console::output::mock();

        let mut prompter = Prompter::new();

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
        console::input::mock("\nnan\n123\n");
        let output = console::output::mock();

        let mut prompter = Prompter::new();

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
        console::input::mock("a\nb\n");
        let output = console::output::mock();

        let mut prompter = Prompter::new();

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

    #[test]
    fn prompt_boolean() {
        console::input::mock("y\nn\n\n\n");
        let output = console::output::mock();

        let mut prompter = Prompter::new();

        assert_eq!(
            prompter.prompt_bool("Works? [y/N]: ", Some(false)).unwrap(),
            Some(true),
        );
        assert_eq!(output.to_string(), "Works? [y/N]: ");

        assert_eq!(
            prompter.prompt_bool("Works? [Y/n]: ", Some(true)).unwrap(),
            Some(false),
        );
        assert_eq!(
            prompter.prompt_bool("Works? [Y/n]: ", Some(true)).unwrap(),
            Some(true),
        );
        assert_eq!(prompter.prompt_bool("Works? [Y/n]: ", None).unwrap(), None,);
    }
}
