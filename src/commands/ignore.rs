use clap::{App, Arg, SubCommand, ArgMatches};
use super::Command;
use super::*;
use super::online::gitignore;
use super::async_trait;

pub struct IgnoreCommand {

}

impl Command for IgnoreCommand {
    fn name(&self) -> String {
        String::from("ignore")
    }

    fn app<'a, 'b>(&self) -> App<'a, 'b> {
        SubCommand::with_name(self.name().as_str())
            .version("1.0")
            .alias("gitignore")
            .about("generates a .gitignore file for the provided languages")
            .help_message("This will manage your .gitignore file using the gitignore.io API to add and update languages.")
            .arg(Arg::with_name("path")
                    .long("path")
                    .help("The path to the .gitignore file you wish to update.")
                    .default_value(".gitignore")
                    .value_name("GITIGNORE")
                    .takes_value(true))
            .arg(Arg::with_name("language")
                    .help("The name of a language which should be added to your .gitignore file.")
                    .multiple(true)
                    .index(1))
    }
}
    
#[async_trait]
impl<F: FileSource, L: Launcher, R: Resolver> CommandRunnable<F, L, R> for IgnoreCommand {
    async fn run<'a>(&self, core: &core::Core<F, L, R>, matches: &ArgMatches<'a>) -> Result<i32, errors::Error> {
        match matches.occurrences_of("language") {
            0 => {
                let languages = gitignore::list().await?;

                for lang in languages {
                    println!("{}", lang);
                }
            },
            _ => {
                let mut original_content: String = String::default();

                let ignore_path = std::path::PathBuf::from(matches.value_of("path").unwrap_or(".gitignore"));

                match core.file_source.read(&ignore_path).await {
                    Ok(content) => original_content = content,
                    Err(_) => {}
                }

                let content = gitignore::add_or_update(original_content.as_str(), matches.values_of("language").unwrap_or_default().collect()).await?;

                core.file_source.write(&ignore_path, content).await?;
            }
        }

        Ok(0)
    }

    async fn complete<'a>(&self, _core: &Core<F, L, R>, completer: &Completer, _matches: &ArgMatches<'a>) {
        match online::gitignore::list().await {
            Ok(langs) => completer.offer_many(langs),
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::core::{Config};
    use clap::ArgMatches;

    #[tokio::test]
    async fn run() {
        let args = ArgMatches::default();
        let cfg = Config::from_str("directory: /dev").unwrap();
        let core = Core::builder().with_config(&cfg).build();

        let cmd = IgnoreCommand{};

        match cmd.run(&core, &args).await {
            Ok(_) => {},
            Err(err) => {
                panic!(err.message())
            }
        }
    }
}