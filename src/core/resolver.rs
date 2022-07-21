use super::features::ALWAYS_OPEN_BEST_MATCH;
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
    #[tracing::instrument(err, skip(self))]
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

    #[tracing::instrument(err, skip(self, name))]
    pub fn get_scratchpad(&self, name: &str) -> Result<Scratchpad, Error> {
        Ok(Scratchpad::new(
            name,
            self.config.get_scratch_directory().join(name),
        ))
    }

    #[tracing::instrument(err, skip(self))]
    pub fn get_current_scratchpad(&self) -> Result<Scratchpad, Error> {
        let time = Local::now();

        self.get_scratchpad(&time.format("%Yw%V").to_string())
    }

    #[tracing::instrument(err, skip(self))]
    pub fn get_current_repo(&self) -> Result<Repo, Error> {
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

    #[tracing::instrument(err, skip(self, path))]
    pub fn get_repo_from_path(&self, path: &std::path::Path) -> Result<Repo, Error> {
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
                repo_from_svc_and_path(&self.config, Some(svc_name), relative_path.strip_prefix(&svc).unwrap_or(relative_path), false)
            },
            Err(e) => Err(errors::system_with_internal(
                "We were unable to determine the repository's fully qualified name.", 
                &format!("Make sure that you are currently within a repository contained within your development directory ('{}').", dev_dir.display()),
                e))
        }
    }

    #[tracing::instrument(err, skip(self, repo))]
    pub fn get_repo(&self, repo: &str) -> Result<Repo, Error> {
        repo_from_str(&self.config, repo, true)
    }

    #[tracing::instrument(err, skip(self, name))]
    pub fn get_best_repo(&self, name: &str) -> Result<Repo, Error> {
        let true_name = self
            .config
            .get_alias(name)
            .unwrap_or_else(|| name.to_string());

        if let Ok(repo) = self.get_repo(&true_name) {
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
                        if let Some(svc) = self.config.get_service(dir.file_name().to_str().unwrap()) {
                            repos.extend(self.get_repos_for(svc)?);
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

    #[tracing::instrument(err, skip(self))]
    pub fn get_repos_for(&self, svc: &Service) -> Result<Vec<Repo>, Error> {
        if !&svc.pattern.split('/').all(|p| p == "*") {
            return Err(errors::user(
                &format!("The glob pattern used for the '{}' service was invalid.", &svc.name),
                "Please ensure that the glob pattern you have used for this service (in your config file) is valid and try again."));
        }

        let path = self.config.get_dev_directory().join(&svc.name);

        let repos = get_child_directories(&path, &svc.pattern)
            .iter()
            .map(|p| self.get_repo_from_path(p))
            .filter_map(|r| r.ok())
            .collect();

        Ok(repos)
    }
}

fn get_service_and_path_from_str(repo: &str) -> (Option<String>, std::path::PathBuf) {
    match repo.split_once(':') {
        Some((svc, path)) => (Some(svc.to_string()), path.into()),
        None => (None, repo.into()),
    }
}

#[cfg_attr(test, mockable)]
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
        Some(svc) => match config.get_service(&svc) {
            Some(svc) => Ok(svc),
            None => Err(errors::user(
                &format!("The service '{}' does not exist.", svc),
                "Please check that you have provided the correct service name, and that the service is present in your Git-Tool config."
            ))
        },
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

#[cfg_attr(test, mockable)]
fn get_child_directories(from: &std::path::PathBuf, pattern: &str) -> Vec<std::path::PathBuf> {
    let depth = pattern.split('/').count();

    get_directory_tree_to_depth(from, depth)
}

#[cfg_attr(test, mockable)]
#[tracing::instrument(skip(from))]
fn get_directory_tree_to_depth(from: &std::path::PathBuf, depth: usize) -> Vec<std::path::PathBuf> {
    if depth == 0 {
        return vec![from.to_owned()];
    }

    from.read_dir()
        .map(|dirs| {
            dirs.filter_map(|dir| match dir {
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

        let svc = resolver.config.get_service("gh").unwrap();

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

        let example = resolver.get_repo("gh:sierrasoftworks/test1").unwrap();
        assert_eq!(example.get_full_name(), "sierrasoftworks/test1");
        assert_eq!(
            example.get_path(),
            resolver
                .config
                .get_dev_directory()
                .join("gh")
                .join("sierrasoftworks")
                .join("test1")
        );
    }

    #[test]
    fn get_repo_exists_absolute() {
        let resolver = get_resolver();

        let example = resolver
            .get_repo_from_path(
                &resolver
                    .config
                    .get_dev_directory()
                    .join("gh")
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
                .join("gh")
                .join("sierrasoftworks")
                .join("test1")
        );
    }

    #[test]
    fn get_repo_new() {
        let resolver = get_resolver();

        let example = resolver.get_repo("gh:sierrasoftworks/test3").unwrap();
        assert_eq!(example.get_full_name(), "sierrasoftworks/test3");
        assert_eq!(
            example.get_path(),
            resolver
                .config
                .get_dev_directory()
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

    #[test]
    fn get_child_directories() {
        let children = super::get_child_directories(&get_dev_dir().join("gh"), "*/*");

        assert_eq!(children.len(), 5);

        assert!(children.iter().any(|p| p
            == &get_dev_dir()
                .join("gh")
                .join("sierrasoftworks")
                .join("test1")));
        assert!(children.iter().any(|p| p
            == &get_dev_dir()
                .join("gh")
                .join("sierrasoftworks")
                .join("test2")));
        assert!(children
            .iter()
            .any(|p| p == &get_dev_dir().join("gh").join("spartan563").join("test1")));
        assert!(children
            .iter()
            .any(|p| p == &get_dev_dir().join("gh").join("spartan563").join("test2")));
    }

    fn get_resolver() -> Resolver {
        let config = Arc::new(Config::for_dev_directory(&get_dev_dir()));

        Resolver::from(config)
    }
}
