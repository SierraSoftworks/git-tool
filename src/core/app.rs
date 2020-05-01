use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct App {
    name: String,
    command: String,
    #[serde(default)]
    args: Vec<String>,
    #[serde(default)]
    environment: Vec<String>,
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
}

pub struct AppBuilder{
    name: String,
    command: String,
    args: Vec<String>,
    environment: Vec<String>,
}

impl Default for AppBuilder {
    fn default() -> Self {
        Self {
            name: Default::default(),
            command: Default::default(),
            args: Default::default(),
            environment: Default::default()
        }
    }
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

impl std::convert::From<&mut AppBuilder> for App {
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