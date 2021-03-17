use super::{errors, Config, Error, Repo, Scratchpad, Service};
use crate::{fs::to_native_path, search};
use chrono::prelude::*;
use std::env;
use std::sync::Arc;

#[cfg(test)]
use mocktopus::macros::*;

#[cfg_attr(test, mockable)]
pub struct Resolver {
    config: Arc<Config>,
}

impl From<Arc<Config>> for Resolver {
    fn from(config: Arc<Config>) -> Self {
        Self { config }
    }
}

#[cfg_attr(test, mockable)]
impl Resolver {
    pub fn get_scratchpads(&self) -> Result<Vec<Scratchpad>, Error> {
        let dirs = self.config.get_scratch_directory().read_dir().map_err(|err| errors::user_with_internal(
            &format!("Could not retrieve the list of directories within your scratchpad directory '{}' due to an OS-level error.", self.config.get_scratch_directory().display()),
            "Check that Git-Tool has permission to access this directory and try again.",
            err
        ))?;

        let mut scratchpads = vec![];
        for dir in dirs {
            let dir_info = dir.map_err(|err| errors::user_with_internal(
                &format!("Could not retrieve information about a directory within your scratchpad directory '{}' due to an OS-level error.", self.config.get_scratch_directory().display()),
                "Check that Git-Tool has permission to access this directory and try again.",
                err
            ))?;

            if dir_info.file_type().map_err(|err| errors::user_with_internal(
                &format!("Could not retrieve information about the directory '{}' due to an OS-level error.", dir_info.path().display()),
                "Check that Git-Tool has permission to access this directory and try again.",
                err
            ))?.is_dir() {
                if let Some(name) = dir_info.file_name().to_str() {
                    scratchpads.push(Scratchpad::new(name, dir_info.path().to_path_buf()));
                }
            }
        }

        Ok(scratchpads)
    }

    pub fn get_scratchpad(&self, name: &str) -> Result<Scratchpad, Error> {
        Ok(Scratchpad::new(
            name.clone(),
            self.config.get_scratch_directory().join(name.clone()),
        ))
    }

    pub fn get_current_scratchpad(&self) -> Result<Scratchpad, Error> {
        let time = Local::now();

        self.get_scratchpad(&time.format("%Yw%V").to_string())
    }

    pub fn get_current_repo(&self) -> Result<Repo, Error> {
        let cwd = env::current_dir().map_err(|err| errors::system_with_internal(
            "Could not determine your current working directory due to an OS-level error.",
            "Please report this issue on GitHub so that we can work with you to investigate the cause and resolve it.",
            err
        ))?;

        match self.get_repo(&cwd) {
            Ok(repo) => Ok(repo),
            Err(e) => Err(errors::user_with_cause(
                &format!("Current directory ('{}') is not a valid repository.", cwd.display()),
                &format!("Make sure that you are currently within a repository contained within your development directory ('{}').", self.config.get_dev_directory().display()),
                e))
        }
    }

    pub fn get_repo(&self, path: &std::path::PathBuf) -> Result<Repo, Error> {
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
            Ok(relative_path) => repo_from_relative_path(&self.config, &relative_path.to_path_buf(), false),
            Err(e) => Err(errors::system_with_internal(
                "We were unable to determine the repository's fully qualified name.", 
                &format!("Make sure that you are currently within a repository contained within your development directory ('{}').", dev_dir.display()),
                e))
        }
    }

    pub fn get_best_repo(&self, name: &str) -> Result<Repo, Error> {
        let true_name =
            std::path::PathBuf::from(self.config.get_alias(name).unwrap_or(name.to_string()));

        match self.get_repo(&true_name) {
            Ok(repo) => return Ok(repo),
            Err(_) => {}
        }

        let all_repos = self.get_repos()?;
        let repos: Vec<&Repo> = search::best_matches_by(name, all_repos.iter(), |r| {
            format!("{}/{}", r.get_domain(), r.get_full_name())
        });

        match repos.len() {
            0 => {
                match repo_from_relative_path(&self.config, &true_name, true) {
                    Ok(repo) => Ok(repo.clone()),
                    Err(_) => Err(errors::user("No matching repository found.", "Please check that you have provided the correct name for the repository and try again."))
                }
            },
            1 => Ok((*repos.first().unwrap()).clone()),
            _ => {
                match repos.iter().find(|r| r.get_full_name() == name) {
                    Some(repo) => Ok((*repo).clone()),
                    None => Err(errors::user("The repository name you provided matched more than one repository.", "Try entering a repository name that is unique, or the fully qualified repository name, to avoid confusion."))
                }
            }
        }
    }

    pub fn get_repos(&self) -> Result<Vec<Repo>, Error> {
        let mut repos = vec![];

        for svc_dir in self.config.get_dev_directory().read_dir().map_err(|err| errors::user_with_internal(
            &format!("Could not retrieve the list of directories within your dev directory '{}' due to an OS-level error.", self.config.get_dev_directory().display()),
            "Check that Git-Tool has permission to access this directory and try again.",
            err
        ))? {
            match svc_dir {
                Ok(dir) => {
                    if dir.file_type().map_err(|err| errors::user_with_internal(
                        &format!("Could not retrieve information about the directory '{}' due to an OS-level error.", dir.path().display()),
                        "Check that Git-Tool has permission to access this directory and try again.",
                        err
                    ))?.is_dir() {
                        match self.config.get_service(dir.file_name().to_str().unwrap()) {
                            Some(svc) => {
                                repos.extend(self.get_repos_for(svc)?);
                            },
                            None => {}
                        }
                    }
                },
                Err(e) => return Err(errors::system_with_internal(
                    "We were unable to access your development directory.",
                    "Please make sure that your development directory exists and that git-tool has permission to access it.",
                    e))
            }
        }

        Ok(repos)
    }

    pub fn get_repos_for(&self, svc: &Service) -> Result<Vec<Repo>, Error> {
        if !svc.get_pattern().split("/").all(|p| p == "*") {
            return Err(errors::user(
                &format!("The glob pattern used for the '{}' service was invalid.", svc.get_domain()),
                "Please ensure that the glob pattern you have used for this service (in your config file) is valid and try again."));
        }

        let path = self.config.get_dev_directory().join(svc.get_domain());

        let repos = get_child_directories(&path, &svc.get_pattern())
            .iter()
            .map(|p| self.get_repo(p))
            .filter(|r| r.is_ok())
            .map(|r| r.unwrap())
            .collect();

        Ok(repos)
    }
}

#[cfg_attr(test, mockable)]
fn service_from_relative_path<'a>(
    config: &'a Config,
    relative_path: &std::path::PathBuf,
) -> Result<&'a Service, Error> {
    if !relative_path.is_relative() {
        return Err(errors::system(
            &format!("The path '{}' used to resolve a repo was not relative.", relative_path.display()),
            "Please report this issue to us on GitHub, including the command you ran, so that we can troubleshoot the problem."));
    }

    let mut components = relative_path.components();
    match components.next() {
        Some(std::path::Component::Normal(name)) => {
            match config.get_service(name.to_str().unwrap()) {
                Some(svc) => Ok(svc),
                None => config.get_default_service().ok_or(errors::user(
                    "No services configured for use with Git Tool.",
                    "Make sure that you have registered a service in your git-tool config using `git-tool config add services/NAME`."
                ))
            }
        },
        _ => Err(errors::user(
            &format!("The path '{}' used to resolve a repo did not start with a service domain name.", relative_path.display()),
            "Make sure that your repository starts with the name of a service, such as 'github.com/sierrasoftworks/git-tool'."))
    }
}

#[cfg_attr(test, mockable)]
fn repo_from_relative_path<'a>(
    config: &'a Config,
    relative_path: &std::path::PathBuf,
    fallback_to_default: bool,
) -> Result<Repo, Error> {
    if !relative_path.is_relative() {
        return Err(errors::system(
            &format!("The path '{}' used to resolve a repo was not relative.", relative_path.display()),
            "Please report this issue to us on GitHub, including the command you ran, so that we can troubleshoot the problem."));
    }

    let svc = service_from_relative_path(config, relative_path)?;
    let name_length = svc.get_pattern().split_terminator("/").count() + 1;
    let mut name_parts: Vec<String> = relative_path
        .components()
        .take(name_length)
        .map(|c| c.as_os_str().to_str().unwrap().to_string())
        .collect();

    let mut true_path = relative_path.to_path_buf();

    if fallback_to_default && !relative_path.starts_with(svc.get_domain()) {
        name_parts.insert(0, svc.get_domain().clone());
        true_path = std::path::PathBuf::from(&svc.get_domain()).join(relative_path);
    }

    if name_parts.len() != name_length {
        Err(errors::user(
            &format!(
                "The service '{}' requires a repository name in the form '{}/{}', but you provided '{}'.",
                svc.get_domain(),
                svc.get_domain(),
                svc.get_pattern(),
                relative_path.display()
            ),
            &format!(
                "Make sure that you are using a repository name which looks like '{}/{}'.",
                svc.get_domain(),
                svc.get_pattern()
            ),
        ))
    } else {
        Ok(Repo::new(
            &name_parts.join("/"),
            to_native_path(config.get_dev_directory().join(true_path)),
        ))
    }
}

#[cfg_attr(test, mockable)]
fn get_child_directories(from: &std::path::PathBuf, pattern: &str) -> Vec<std::path::PathBuf> {
    let depth = pattern.split("/").count();

    get_directory_tree_to_depth(from, depth)
}

#[cfg_attr(test, mockable)]
fn get_directory_tree_to_depth(from: &std::path::PathBuf, depth: usize) -> Vec<std::path::PathBuf> {
    if depth == 0 {
        return vec![from.clone()];
    }

    from.read_dir()
        .map(|dirs| {
            dirs.map(|dir| match dir {
                Ok(d) => match d.file_type() {
                    Ok(ft) => {
                        if ft.is_dir() {
                            Some(d.path())
                        } else {
                            None
                        }
                    }
                    Err(_) => None,
                },
                Err(_) => None,
            })
            .filter(|d| d.is_some())
            .map(|d| d.unwrap())
            .flat_map(|d| get_directory_tree_to_depth(&d, depth - 1))
            .collect()
        })
        .unwrap()
}

#[cfg(test)]
mod tests {
    use super::super::Target;
    use super::{Config, Resolver};
    use crate::test::get_dev_dir;
    use chrono::prelude::*;
    use std::{path, sync::Arc};

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
                .join("github.com")
                .join("sierrasoftworks")
                .join("test1")
        );
    }

    #[test]
    fn get_repos_for() {
        let resolver = get_resolver();

        let svc = resolver.config.get_service("github.com").unwrap();

        let results = resolver.get_repos_for(svc).unwrap();
        assert_eq!(results.len(), 5);

        let example = results
            .iter()
            .find(|r| r.get_full_name() == "sierrasoftworks/test1")
            .unwrap();
        assert_eq!(
            example.get_path(),
            get_dev_dir()
                .join("github.com")
                .join("sierrasoftworks")
                .join("test1")
        );
    }

    #[test]
    fn get_best_repo() {
        let resolver = get_resolver();

        let example = resolver.get_best_repo("ghspt1").unwrap();
        assert_eq!(example.get_full_name(), "spartan563/test1");
    }

    #[test]
    fn get_repo_exists() {
        let resolver = get_resolver();

        let example = resolver
            .get_repo(&path::PathBuf::from("github.com/sierrasoftworks/test1"))
            .unwrap();
        assert_eq!(example.get_full_name(), "sierrasoftworks/test1");
        assert_eq!(
            example.get_path(),
            resolver
                .config
                .get_dev_directory()
                .join("github.com")
                .join("sierrasoftworks")
                .join("test1")
        );
    }

    #[test]
    fn get_repo_exists_absolute() {
        let resolver = get_resolver();

        let example = resolver
            .get_repo(
                &resolver
                    .config
                    .get_dev_directory()
                    .join("github.com")
                    .join("sierrasoftworks")
                    .join("test1"),
            )
            .unwrap();
        assert_eq!(example.get_full_name(), "sierrasoftworks/test1");
        assert_eq!(
            example.get_path(),
            resolver
                .config
                .get_dev_directory()
                .join("github.com")
                .join("sierrasoftworks")
                .join("test1")
        );
    }

    #[test]
    fn get_repo_new() {
        let resolver = get_resolver();

        let example = resolver
            .get_repo(&path::PathBuf::from("github.com/sierrasoftworks/test3"))
            .unwrap();
        assert_eq!(example.get_full_name(), "sierrasoftworks/test3");
        assert_eq!(
            example.get_path(),
            resolver
                .config
                .get_dev_directory()
                .join("github.com")
                .join("sierrasoftworks")
                .join("test3")
        );
    }

    #[test]
    fn get_best_repo_default_service() {
        let resolver = get_resolver();

        let example = resolver.get_best_repo("sierrasoftworks/test3").unwrap();
        assert_eq!(example.get_domain(), "github.com");
        assert_eq!(example.get_full_name(), "sierrasoftworks/test3");
        assert_eq!(
            example.get_path(),
            get_dev_dir()
                .join("github.com")
                .join("sierrasoftworks")
                .join("test3")
        );
    }

    #[test]
    fn get_best_repo_default_service_multiple_matches() {
        let resolver = get_resolver();

        let example = resolver.get_best_repo("sierrasoftworks/test1").unwrap();
        assert_eq!(example.get_domain(), "github.com");
        assert_eq!(example.get_full_name(), "sierrasoftworks/test1");
        assert_eq!(
            example.get_path(),
            get_dev_dir()
                .join("github.com")
                .join("sierrasoftworks")
                .join("test1")
        );
    }

    #[test]
    fn get_child_directories() {
        let children = super::get_child_directories(&get_dev_dir().join("github.com"), "*/*");

        assert_eq!(children.len(), 5);

        assert!(children.iter().any(|p| p
            == &get_dev_dir()
                .join("github.com")
                .join("sierrasoftworks")
                .join("test1")));
        assert!(children.iter().any(|p| p
            == &get_dev_dir()
                .join("github.com")
                .join("sierrasoftworks")
                .join("test2")));
        assert!(children.iter().any(|p| p
            == &get_dev_dir()
                .join("github.com")
                .join("spartan563")
                .join("test1")));
        assert!(children.iter().any(|p| p
            == &get_dev_dir()
                .join("github.com")
                .join("spartan563")
                .join("test2")));
    }

    fn get_resolver() -> Resolver {
        let config = Arc::new(Config::for_dev_directory(&get_dev_dir()));

        Resolver::from(config)
    }
}
