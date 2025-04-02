use std::{
    io::{Read, Write},
    sync::Arc,
};
use tracing_batteries::prelude::*;

use crate::console::ConsoleProvider;

use super::Error;

pub struct Prompter {
    console: Arc<dyn ConsoleProvider + Send + Sync>,
}

impl Prompter {
    pub fn new(console: Arc<dyn ConsoleProvider + Send + Sync>) -> Self {
        Self { console }
    }

    #[tracing::instrument(err, skip(self, validate))]
    pub fn prompt<V>(&mut self, message: &str, validate: V) -> Result<Option<String>, Error>
    where
        V: Fn(&str) -> bool,
    {
        let mut writer = self.console.output();
        let mut reader = self.console.input();

        for _i in 0..3 {
            write!(writer, "{message}")?;
            writer.flush()?;

            let line = Self::read_line(&mut reader)?;
            if line.is_empty() {
                return Ok(None);
            }

            if validate(line.trim()) {
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

    // NOTE: We use unbuffered reads here since the prompter is stateless and needs to avoid
    // consuming output from future prompts.
    #[allow(clippy::unbuffered_bytes)]
    fn read_line<R: Read>(reader: R) -> Result<String, Error> {
        let mut bytes = Vec::with_capacity(128);

        for byte in reader.bytes() {
            match byte {
                Err(e) => return Err(e.into()),
                Ok(char) => {
                    bytes.push(char);

                    if char == b'\n' {
                        break;
                    }
                }
            }
        }

        Ok(String::from_utf8(bytes)?)
    }
}

#[cfg(test)]
mod tests {
    use crate::engine::Core;

    #[test]
    fn prompt_for_any() {
        let console = crate::console::mock_with_input("123\n");

        let core = Core::builder()
            .with_default_config()
            .with_console(console.clone())
            .build();

        let mut prompter = core.prompter();

        assert_eq!(
            prompter
                .prompt("Enter a number: ", |l| {
                    let n: Option<u32> = l.parse().ok();
                    n.is_some()
                })
                .unwrap(),
            Some("123".into()),
        );

        assert_eq!(console.to_string(), "Enter a number: ");
    }

    #[test]
    fn prompt_eof() {
        let console = crate::console::mock_with_input("");

        let core = Core::builder()
            .with_default_config()
            .with_console(console.clone())
            .build();

        let mut prompter = core.prompter();

        assert_eq!(
            prompter
                .prompt("Enter a number: ", |l| {
                    let n: Option<u32> = l.parse().ok();
                    n.is_some()
                })
                .unwrap(),
            None,
        );

        assert_eq!(console.to_string(), "Enter a number: ");
    }

    #[test]
    fn prompt_retry() {
        let console = crate::console::mock_with_input("\nnan\n123\n");

        let core = Core::builder()
            .with_default_config()
            .with_console(console.clone())
            .build();

        let mut prompter = core.prompter();
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
            console.to_string(),
            "Enter a number: Enter a number: Enter a number: "
        );
    }

    #[test]
    fn prompt_multiple() {
        let console = crate::console::mock_with_input("a\nb\n");

        let core = Core::builder()
            .with_default_config()
            .with_console(console.clone())
            .build();

        let mut prompter = core.prompter();

        assert_eq!(
            prompter.prompt("First prompt: ", |_| true).unwrap(),
            Some("a".into()),
        );

        assert_eq!(
            prompter.prompt("Second prompt: ", |_| true).unwrap(),
            Some("b".into()),
        );

        assert_eq!(console.to_string(), "First prompt: Second prompt: ");
    }

    #[test]
    fn prompt_boolean() {
        let console = crate::console::mock_with_input("y\nn\n\n\n");

        let core = Core::builder()
            .with_default_config()
            .with_console(console.clone())
            .build();

        let mut prompter = core.prompter();

        assert_eq!(
            prompter.prompt_bool("Works? [y/N]: ", Some(false)).unwrap(),
            Some(true),
        );
        assert_eq!(console.to_string(), "Works? [y/N]: ");

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
