use std::io::{BufRead, BufReader};

use crate::core::{Input, Output};

use super::Error;

pub fn prompt<V, I, O>(
    input: &I,
    output: &O,
    message: &str,
    validate: V,
) -> Result<Option<String>, Error>
where
    V: Fn(&str) -> bool,
    I: Input,
    O: Output,
{
    let mut writer = output.writer();
    let mut reader = BufReader::new(input.reader());
    let mut line = String::default();

    for _i in 0..3 {
        write!(writer, "{}", message)?;
        writer.flush()?;

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

        assert_eq!(
            prompt(&input, &output, "Enter a number: ", |l| {
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

        assert_eq!(
            prompt(&input, &output, "Enter a number: ", |l| {
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

        assert_eq!(
            prompt(&input, &output, "Enter a number: ", |l| {
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
}
