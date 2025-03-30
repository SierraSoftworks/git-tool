use super::features::ALWAYS_OPEN_BEST_MATCH;
use super::{errors, Config, Error, Repo, Scratchpad, Service, TempTarget};
use crate::{fs::to_native_path, search};
use chrono::prelude::*;
use std::env;
use std::sync::Arc;
use tracing_batteries::prelude::*;

use crate::fs::{get_child_directories, resolve_directories};
#[cfg(test)]
use mockall::automock;

#[cfg_attr(test, automock)]
pub trait Resolver: Send + Sync {
    fn get_temp(&self, keep: bool) -> Result<TempTarget, Error>;

    fn get_scratchpads(&self) -> Result<Vec<Scratchpad>, Error>;
    fn get_scratchpad(&self, name: &str) -> Result<Scratchpad, Error>;
    fn get_current_scratchpad(&self) -> Result<Scratchpad, Error>;

    fn get_repos(&self) -> Result<Vec<Repo>, Error>;
    fn get_repos_for(&self, service: &Service) -> Result<Vec<Repo>, Error>;
    fn get_best_repo(&self, name: &str) -> Result<Repo, Error>;
    fn get_current_repo(&self) -> Result<Repo, Error>;
}

pub fn resolver(config: Arc<Config>) -> Arc<dyn Resolver + Send + Sync> {
    Arc::new(TrueResolver { config })
}

struct TrueResolver {
    config: Arc<Config>,
}

impl From<Arc<Config>> for TrueResolver {
    fn from(config: Arc<Config>) -> Self {
        Self { config }
    }
}

impl Resolver for TrueResolver {
    #[tracing::instrument(err, skip(self))]
    fn get_temp(&self, keep: bool) -> Result<TempTarget, Error> {
        TempTarget::new(keep)
    }

    #[tracing::instrument(err, skip(self))]
    fn get_scratchpads(&self) -> Result<Vec<Scratchpad>, Error> {
        Ok(get_child_directories(&self.config.get_scratch_directory())?
            .into_iter()
            .filter_map(|p| {
                p.file_name()
                    .and_then(|f| f.to_str())
                    .map(|name| Scratchpad::new(name, p.to_path_buf()))
            })
            .collect())
    }

    #[tracing::instrument(err, skip(self, name))]
    fn get_scratchpad(&self, name: &str) -> Result<Scratchpad, Error> {
        if name.starts_with('^') || name.starts_with('~') {
            let delta = name[1..].parse::<u64>().map_err(|err| {
                errors::user(
                    &format!(
                        "Could not parse the offset expression '{}' into a valid week offset: {}.",
                        &name, err,
                    ),
                    "Please provide a valid number of weeks to go back in time.",
                )
            })?;

            let time = Local::now() - chrono::Duration::days(delta as i64 * 7);

            return self.get_scratchpad(&time.format("%Yw%V").to_string());
        }

        Ok(Scratchpad::new(
            name,
            self.config.get_scratch_directory().join(name),
        ))
    }

    #[tracing::instrument(err, skip(self))]
    fn get_current_scratchpad(&self) -> Result<Scratchpad, Error> {
        let time = Local::now();

        self.get_scratchpad(&time.format("%Yw%V").to_string())
    }

    #[tracing::instrument(err, skip(self))]
    fn get_repos(&self) -> Result<Vec<Repo>, Error> {
        let mut repos = vec![];

        for svc in self.config.get_services() {
            repos.extend(self.get_repos_for(svc)?);
        }

        Ok(repos)
    }

    #[tracing::instrument(err, skip(self, svc), fields(service=%svc.name))]
    fn get_repos_for(&self, svc: &Service) -> Result<Vec<Repo>, Error> {
        if !&svc.pattern.split('/').all(|p| p == "*") {
            return Err(errors::user(
                &format!("The glob pattern used for the '{}' service was invalid.", &svc.name),
                "Please ensure that the glob pattern you have used for this service (in your config file) is valid and try again."));
        }

        let path = self.config.get_dev_directory().join(&svc.name);

        let repos = resolve_directories(&path, &svc.pattern)?
            .iter()
            .map(|p| self.get_repo_from_path(p))
            .filter_map(|r| r.ok())
            .collect();

        Ok(repos)
    }

    #[tracing::instrument(err, skip(self, name))]
    fn get_best_repo(&self, name: &str) -> Result<Repo, Error> {
        let true_name = self
            .config
            .get_alias(name)
            .unwrap_or_else(|| name.to_string());

        if let Ok(repo) = repo_from_str(&self.config, &true_name, true) {
            return Ok(repo);
        }

        let all_repos = self.get_repos()?;
        let repos: Vec<&Repo> = search::best_matches_by(name, all_repos.iter(), |r| {
            format!("{}:{}", &r.service, r.get_full_name())
        });

        match repos.len() {
            0 => {
                match repo_from_str(&self.config, &true_name, true) {
                    Ok(repo) => Ok(repo),
                    Err(_) => Err(errors::user("No matching repository found.", "Please check that you have provided the correct name for the repository and try again."))
                }
            },
            1 => Ok((*repos.first().unwrap()).clone()),
            _ if self.config.get_features().has(ALWAYS_OPEN_BEST_MATCH) => Ok((*repos.first().unwrap()).clone()),
            _ => {
                match repos.iter().find(|r| r.get_full_name() == name) {
                    Some(repo) => Ok((*repo).clone()),
                    None => Err(errors::user("The repository name you provided matched more than one repository.", "Try entering a repository name that is unique, or the fully qualified repository name, to avoid confusion."))
                }
            }
        }
    }

    #[tracing::instrument(err, skip(self))]
    fn get_current_repo(&self) -> Result<Repo, Error> {
        let cwd = env::current_dir().map_err(|err| errors::system_with_internal(
            "Could not determine your current working directory due to an OS-level error.",
            "Please report this issue on GitHub so that we can work with you to investigate the cause and resolve it.",
            err
        ))?;

        match self.get_repo_from_path(&cwd) {
            Ok(repo) => Ok(repo),
            Err(e) => Err(errors::user_with_cause(
                &format!("Current directory ('{}') is not a valid repository.", cwd.display()),
                &format!("Make sure that you are currently within a repository contained within your development directory ('{}').", self.config.get_dev_directory().display()),
                e))
        }
    }
}

impl TrueResolver {
    fn get_repo_from_path(&self, path: &std::path::Path) -> Result<Repo, Error> {
        debug!("Constructing repo object from path '{}'", path.display());
        let dev_dir = self.config.get_dev_directory().canonicalize().map_err(|err| errors::user_with_internal(
            &format!("Could not determine the canonical path for your dev directory '{}' due to an OS-level error.", self.config.get_dev_directory().display()),
            "Check that the directory exists and that Git-Tool has permission to access it.",
            err
        ))?;
        let dir = if path.is_absolute() {
            path.canonicalize().map_err(|err| errors::user_with_internal(
                &format!("Could not determine the canonical path for the directory '{}' due to an OS-level error.", path.display()),
                "Check that the directory exists and that Git-Tool has permission to access it.",
                err
            ))?
        } else {
            dev_dir.join(path)
        };

        if !dir.starts_with(&dev_dir) || dir == dev_dir {
            return Err(errors::user(
                "Current directory is not a valid repository.",
                &format!("Make sure that you are currently within a repository contained within your development directory ('{}').", dev_dir.display())));
        }

        match dir.strip_prefix(&dev_dir) {
            Ok(relative_path) => {
                let svc = relative_path.components().next().ok_or_else(|| errors::user(
                    "Current directory is not a valid repository.",
                    &format!("Make sure that you are currently within a repository contained within your development directory ('{}').", dev_dir.display())))?;
                let svc_name = svc.as_os_str().to_string_lossy().to_string();
                repo_from_svc_and_path(&self.config, Some(svc_name), relative_path.strip_prefix(svc).unwrap_or(relative_path), false)
            },
            Err(e) => Err(errors::system_with_internal(
                "We were unable to determine the repository's fully qualified name.",
                &format!("Make sure that you are currently within a repository contained within your development directory ('{}').", dev_dir.display()),
                e))
        }
    }
}

fn get_service_and_path_from_str(repo: &str) -> (Option<String>, std::path::PathBuf) {
    match repo.split_once(':') {
        Some((svc, path)) => (Some(svc.to_string()), path.into()),
        None => (None, repo.into()),
    }
}

fn repo_from_str(config: &Config, repo: &str, fallback_to_default: bool) -> Result<Repo, Error> {
    let (svc, path) = get_service_and_path_from_str(repo);
    repo_from_svc_and_path(config, svc, &path, fallback_to_default)
}

fn repo_from_svc_and_path(
    config: &Config,
    svc: Option<String>,
    path: &std::path::Path,
    fallback_to_default: bool,
) -> Result<Repo, Error> {
    let svc = match svc {
        Some(svc) => config.get_service(&svc),
        None if fallback_to_default => config.get_default_service().ok_or_else(|| errors::user(
            "No services configured for use with Git Tool.",
            "Make sure that you have registered a service in your Git-Tool config using `git-tool config add services/NAME`."
        )),
        None => Err(errors::user(
            &format!("The path '{}' used to resolve a repo did not start with a service namespace.", path.display()),
            "Make sure that your repository starts with the name of a service, such as 'gh:sierrasoftworks/git-tool'."))
    }?;

    let name_length = svc.pattern.split_terminator('/').count();
    let name_parts: Vec<String> = path
        .components()
        .take(name_length)
        .map(|c| c.as_os_str().to_str().unwrap().to_string())
        .collect();

    let true_path = std::path::PathBuf::from(&svc.name).join(path);

    if name_parts.len() != name_length {
        Err(errors::user(
            &format!(
                "The service '{}' requires a repository name in the form '{}', but you provided '{}'.",
                &svc.name,
                &svc.pattern,
                path.display()
            ),
            &format!(
                "Make sure that you are using a repository name which looks like '{}:{}'.",
                &svc.name,
                &svc.pattern
            ),
        ))
    } else {
        Ok(Repo::new(
            &format!("{}:{}", &svc.name, &name_parts.join("/")),
            to_native_path(config.get_dev_directory().join(true_path)),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::super::Target;
    use super::{resolver, Config, Resolver};
    use crate::core::resolver::TrueResolver;
    use crate::test::get_dev_dir;
    use chrono::prelude::*;
    use std::sync::Arc;

    #[test]
    fn get_scratchpads() {
        let resolver = get_resolver();

        let results = resolver.get_scratchpads().unwrap();
        assert_eq!(results.len(), 3);
        assert!(results.iter().any(|r| r.get_name() == "2019w15"));
        assert!(results.iter().any(|r| r.get_name() == "2019w16"));
        assert!(results.iter().any(|r| r.get_name() == "2019w27"));

        let example = results.iter().find(|r| r.get_name() == "2019w15").unwrap();
        assert_eq!(
            example.get_path(),
            get_dev_dir().join("scratch").join("2019w15")
        );
    }

    #[test]
    fn get_scratchpad_existing() {
        let resolver = get_resolver();

        let example = resolver.get_scratchpad("2019w15").unwrap();
        assert_eq!(
            example.get_path(),
            get_dev_dir().join("scratch").join("2019w15")
        );
    }

    #[test]
    fn get_scratchpad_new() {
        let resolver = get_resolver();

        let example = resolver.get_scratchpad("2019w10").unwrap();
        assert_eq!(
            example.get_path(),
            get_dev_dir().join("scratch").join("2019w10")
        );
    }

    #[test]
    fn get_scratchpad_offset() {
        let resolver = get_resolver();

        let example = resolver.get_scratchpad("^0").unwrap();
        assert_eq!(
            example.get_path(),
            get_dev_dir()
                .join("scratch")
                .join(Local::now().format("%Yw%V").to_string())
        );

        let example = resolver.get_scratchpad("^1").unwrap();
        assert_eq!(
            example.get_path(),
            get_dev_dir().join("scratch").join(
                (Local::now() - chrono::Duration::days(7))
                    .format("%Yw%V")
                    .to_string()
            )
        );

        let example = resolver.get_scratchpad("^5").unwrap();
        assert_eq!(
            example.get_path(),
            get_dev_dir().join("scratch").join(
                (Local::now() - chrono::Duration::days(7 * 5))
                    .format("%Yw%V")
                    .to_string()
            )
        );

        assert!(resolver.get_scratchpad("^not-a-number").is_err());
        assert!(resolver.get_scratchpad("^-1").is_err());
    }

    #[test]
    fn get_current_scratchpad() {
        let resolver = get_resolver();

        let time = Local::now();
        let name = time.format("%Yw%V").to_string();

        let example = resolver.get_current_scratchpad().unwrap();
        assert_eq!(example.get_name(), name);
        assert_eq!(example.get_path(), get_dev_dir().join("scratch").join(name));
    }

    #[test]
    fn get_repos() {
        let resolver = get_resolver();

        let results = resolver.get_repos().unwrap();
        assert_eq!(results.len(), 7);

        let example = results
            .iter()
            .find(|r| r.get_full_name() == "sierrasoftworks/test1")
            .unwrap();
        assert_eq!(
            example.get_path(),
            get_dev_dir()
                .join("gh")
                .join("sierrasoftworks")
                .join("test1")
        );
    }

    #[test]
    fn get_repos_for() {
        let resolver = get_resolver();
        let config = Config::default();

        let svc = config.get_service("gh").unwrap();

        let results = resolver.get_repos_for(svc).unwrap();
        assert_eq!(results.len(), 5);

        let example = results
            .iter()
            .find(|r| r.get_full_name() == "sierrasoftworks/test1")
            .unwrap();
        assert_eq!(
            example.get_path(),
            get_dev_dir()
                .join("gh")
                .join("sierrasoftworks")
                .join("test1")
        );
    }

    #[test]
    fn get_best_repo() {
        let resolver = get_resolver();

        let example = resolver.get_best_repo("gh:spt1").unwrap();
        assert_eq!(example.get_full_name(), "spartan563/test1");
    }

    #[test]
    fn get_repo_exists() {
        let resolver = get_resolver();

        let example = resolver.get_best_repo("gh:sierrasoftworks/test1").unwrap();
        assert_eq!(example.get_full_name(), "sierrasoftworks/test1");
        assert_eq!(
            example.get_path(),
            get_dev_dir()
                .join("gh")
                .join("sierrasoftworks")
                .join("test1")
        );
    }

    #[test]
    fn get_repo_exists_absolute() {
        let resolver = TrueResolver {
            config: Arc::new(Config::for_dev_directory(&get_dev_dir())),
        };

        let example = resolver
            .get_repo_from_path(
                &get_dev_dir()
                    .join("gh")
                    .join("sierrasoftworks")
                    .join("test1"),
            )
            .unwrap();
        assert_eq!(example.get_full_name(), "sierrasoftworks/test1");
        assert_eq!(
            example.get_path(),
            get_dev_dir()
                .join("gh")
                .join("sierrasoftworks")
                .join("test1")
        );
    }

    #[test]
    fn get_best_repo_new() {
        let resolver = get_resolver();

        let example = resolver.get_best_repo("gh:sierrasoftworks/test3").unwrap();
        assert_eq!(example.get_full_name(), "sierrasoftworks/test3");
        assert_eq!(
            example.get_path(),
            get_dev_dir()
                .join("gh")
                .join("sierrasoftworks")
                .join("test3")
        );
    }

    #[test]
    fn get_best_repo_default_service() {
        let resolver = get_resolver();

        let example = resolver.get_best_repo("sierrasoftworks/test3").unwrap();
        assert_eq!(&example.service, "gh");
        assert_eq!(example.get_full_name(), "sierrasoftworks/test3");
        assert_eq!(
            example.get_path(),
            get_dev_dir()
                .join("gh")
                .join("sierrasoftworks")
                .join("test3")
        );
    }

    #[test]
    fn get_best_repo_default_service_multiple_matches() {
        let resolver = get_resolver();

        let example = resolver.get_best_repo("sierrasoftworks/test1").unwrap();
        assert_eq!(&example.service, "gh");
        assert_eq!(example.get_full_name(), "sierrasoftworks/test1");
        assert_eq!(
            example.get_path(),
            get_dev_dir()
                .join("gh")
                .join("sierrasoftworks")
                .join("test1")
        );
    }

    fn get_resolver() -> Arc<dyn Resolver + Send + Sync> {
        let config = Arc::new(Config::for_dev_directory(&get_dev_dir()));

        resolver(config)
    }
}
