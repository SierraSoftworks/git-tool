use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct App {
    name: String,
    command: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    args: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    environment: Vec<String>,

    /// Literal environment variable overrides provided at launch time. These are
    /// applied verbatim (they are never run through the template engine) and are
    /// not part of the persisted configuration.
    #[serde(skip)]
    overrides: Vec<(String, String)>,
}

impl App {
    pub fn builder() -> AppBuilder {
        AppBuilder::default()
    }

    pub fn get_name(&self) -> &str {
        self.name.as_str()
    }

    pub fn get_command(&self) -> &str {
        self.command.as_str()
    }

    pub fn get_args(&self) -> Vec<String> {
        self.args.clone()
    }

    pub fn get_environment(&self) -> Vec<String> {
        self.environment.clone()
    }

    /// Returns a copy of this application with the provided literal environment
    /// overrides attached. The overrides are applied verbatim at launch time and
    /// take precedence over any configured `environment` entries.
    pub fn with_overrides(&self, overrides: Vec<(String, String)>) -> App {
        let mut app = self.clone();
        app.overrides = overrides;
        app
    }

    pub fn get_overrides(&self) -> &[(String, String)] {
        &self.overrides
    }
}

impl Display for App {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: ", &self.name)?;
        if !self.environment.is_empty() {
            write!(f, "{} ", &self.environment.join(" "))?;
        }
        if !self.overrides.is_empty() {
            let overrides = self
                .overrides
                .iter()
                .map(|(k, v)| format!("{k}={v}"))
                .collect::<Vec<_>>()
                .join(" ");
            write!(f, "{overrides} ")?;
        }

        write!(f, "{}", &self.command)?;
        if !self.args.is_empty() {
            write!(f, " {}", &self.args.join(" "))?;
        }

        Ok(())
    }
}

#[derive(Default, Debug)]
pub struct AppBuilder {
    name: String,
    command: String,
    args: Vec<String>,
    environment: Vec<String>,
}

impl AppBuilder {
    pub fn with_name(&mut self, name: &str) -> &mut AppBuilder {
        self.name = String::from(name);

        self
    }

    pub fn with_command(&mut self, command: &str) -> &mut AppBuilder {
        self.command = String::from(command);

        self
    }

    #[allow(dead_code)]
    pub fn with_args(&mut self, args: Vec<&str>) -> &mut AppBuilder {
        self.args = args.iter().map(|x| String::from(*x)).collect();

        self
    }

    #[allow(dead_code)]
    pub fn with_environment(&mut self, env: Vec<&str>) -> &mut AppBuilder {
        self.environment = env.iter().map(|x| String::from(*x)).collect();

        self
    }
}

impl From<&mut AppBuilder> for App {
    fn from(builder: &mut AppBuilder) -> Self {
        if builder.name.is_empty() {
            panic!("cannot construct an app with an empty name")
        }

        if builder.command.is_empty() {
            panic!("cannot construct an app with an empty command")
        }

        Self {
            name: builder.name.clone(),
            command: builder.command.clone(),
            args: builder.args.clone(),
            environment: builder.environment.clone(),
            overrides: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::App;

    #[test]
    fn app_builder() {
        let app: App = App::builder()
            .with_name("test")
            .with_command("/bin/sh")
            .with_args(vec!["-c", "echo $TEST"])
            .with_environment(vec!["TEST=test"])
            .into();

        assert_eq!(app.get_name(), "test");
        assert_eq!(app.get_command(), "/bin/sh");
        assert_eq!(app.get_args(), vec!["-c", "echo $TEST"]);
        assert_eq!(app.get_environment(), vec!["TEST=test"]);
    }
}
