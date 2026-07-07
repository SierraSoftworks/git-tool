use std::env;

use super::{ResolveMany, Resolver, TrueResolver};
use crate::engine::features::ALWAYS_OPEN_BEST_MATCH;
use crate::engine::{Config, Core, Identifier, Repo, Service};
use crate::fs::{resolve_directories, to_native_path};
use crate::search;
use human_errors::{OptionExt, ResultExt};
use tracing_batteries::prelude::*;

/// Resolves the repository containing the current working directory.
impl Resolver<(), Repo> for Core {
    fn resolve(&self, source: ()) -> Result<Repo, human_errors::Error> {
        self.resolve_with_events(source, "current")
    }
}

/// Resolves the best matching repository for a (possibly partial) identifier,
/// taking aliases and the configured default service into account.
impl Resolver<&Identifier, Repo> for Core {
    fn resolve(&self, source: &Identifier) -> Result<Repo, human_errors::Error> {
        self.resolve_with_events(source, "name")
    }
}

/// Resolves the best matching repository for a raw name, parsing it into an
/// [`Identifier`] first.
impl Resolver<&str, Repo> for Core {
    fn resolve(&self, source: &str) -> Result<Repo, human_errors::Error> {
        let identifier: Identifier = source.parse()?;
        self.resolve(&identifier)
    }
}

/// Enumerates every repository within the development directory.
impl ResolveMany<(), Repo> for Core {
    fn resolve_many(&self, source: ()) -> Result<Vec<Repo>, human_errors::Error> {
        self.resolve_many_with_events(source, "all")
    }
}

/// Enumerates the repositories belonging to a specific service.
impl ResolveMany<&Service, Repo> for Core {
    fn resolve_many(&self, source: &Service) -> Result<Vec<Repo>, human_errors::Error> {
        self.resolve_many_with_events(source, "service")
    }
}

impl Resolver<(), Repo> for TrueResolver {
    #[tracing::instrument(err, skip(self, _source))]
    fn resolve(&self, _source: ()) -> Result<Repo, human_errors::Error> {
        let cwd = env::current_dir().map_err(|err| human_errors::wrap_system(
                err,
                "Could not determine your current working directory due to an OS-level error.",
                &["Please report this issue on GitHub so that we can work with you to investigate the cause and resolve it."],
            ))?;

        self.repo_from_path(&cwd).wrap_user_err(
            format!(
                "Current directory ('{}') is not a valid repository.",
                cwd.display()
            ),
            &["Make sure that you are currently within a repository contained within your development directory."],
        )
    }
}

impl Resolver<&Identifier, Repo> for TrueResolver {
    #[tracing::instrument(err, skip(self, identifier))]
    fn resolve(&self, identifier: &Identifier) -> Result<Repo, human_errors::Error> {
        let true_name = self
            .config
            .get_alias(&identifier.path)
            .unwrap_or_else(|| identifier.to_string());

        if let Ok(repo) = repo_from_str(&self.config, &true_name, true) {
            return Ok(repo);
        }

        let all_repos: Vec<Repo> = match &identifier.scope {
            ns if ns.is_empty() => self.resolve_many(())?,
            ns => self.resolve_many(self.config.get_service(ns).map_err(|_| {
                human_errors::user(
                    format!(
                        "The service '{}' used to resolve a repo was not present in your config.",
                        ns
                    ),
                    &["Try adding the namespace to your configuration as a supported service."],
                )
            })?)?,
        };

        let full_name = identifier.to_string();

        let repos: Vec<&Repo> = search::best_matches_by(&full_name, all_repos.iter(), |r| {
            format!("{}:{}", &r.service, r.get_full_name())
        });

        match repos.len() {
            0 => match repo_from_str(&self.config, &true_name, true) {
                Ok(repo) => Ok(repo),
                Err(_) => Err(human_errors::user(
                    format!(
                        "None of your local repositories matched '{full_name}'. Please check that you have provided the correct name for the repository or try cloning it with 'gt open {full_name}'."
                    ),
                    &["Verify the repository name is correct."],
                )),
            },
            1 => Ok((*repos.first().unwrap()).clone()),
            _ if self.config.get_features().has(ALWAYS_OPEN_BEST_MATCH) => {
                Ok((*repos.first().unwrap()).clone())
            }
            _ => match repos.iter().find(|r| r.get_full_name() == full_name) {
                Some(repo) => Ok((*repo).clone()),
                None => Err(human_errors::user(
                    "The repository name you provided matched more than one repository.",
                    &[
                        "Try entering a repository name that is unique, or the fully qualified repository name, to avoid confusion.",
                    ],
                )),
            },
        }
    }
}

impl ResolveMany<(), Repo> for TrueResolver {
    #[tracing::instrument(err, skip(self, _source))]
    fn resolve_many(&self, _source: ()) -> Result<Vec<Repo>, human_errors::Error> {
        let mut repos = vec![];

        for svc in self.config.get_services() {
            repos.extend(self.resolve_many(svc.as_ref())?);
        }

        Ok(repos)
    }
}

impl ResolveMany<&Service, Repo> for TrueResolver {
    #[tracing::instrument(err, skip(self, svc), fields(service=%svc.name))]
    fn resolve_many(&self, svc: &Service) -> Result<Vec<Repo>, human_errors::Error> {
        if !&svc.pattern.split('/').all(|p| p == "*") {
            return Err(human_errors::user(
                format!(
                    "The glob pattern used for the '{}' service was invalid.",
                    &svc.name
                ),
                &[
                    "Please ensure that the glob pattern you have used for this service (in your config file) is valid and try again.",
                ],
            ));
        }

        let path = self.config.get_dev_directory().join(&svc.name);

        let repos = resolve_directories(&path, &svc.pattern)?
            .iter()
            .map(|p| self.repo_from_path(p))
            .collect::<Result<Vec<Repo>, human_errors::Error>>()?;

        Ok(repos)
    }
}

impl TrueResolver {
    /// Constructs the [`Repo`] which a directory within the development
    /// directory corresponds to. This is shared by the current-repo resolution
    /// (which starts from the working directory) and repository enumeration
    /// (which starts from the directories on disk).
    pub(super) fn repo_from_path(
        &self,
        path: &std::path::Path,
    ) -> Result<Repo, human_errors::Error> {
        debug!("Constructing repo object from path '{}'", path.display());
        let dev_dir = self.config.get_dev_directory().canonicalize().wrap_user_err(
            format!("Could not determine the canonical path for your dev directory '{}' due to an OS-level error.", self.config.get_dev_directory().display()),
            &["Check that the directory exists and that Git-Tool has permission to access it."],
        )?;
        let dir = if path.is_absolute() {
            path.canonicalize().wrap_user_err(
                format!("Could not determine the canonical path for the directory '{}' due to an OS-level error.", path.display()),
                &["Check that the directory exists and that Git-Tool has permission to access it."],
            )?
        } else {
            dev_dir.join(path)
        };

        if !dir.starts_with(&dev_dir) || dir == dev_dir {
            return Err(human_errors::user(
                "Current directory is not a valid repository.",
                &[
                    "Make sure that you are currently within a repository contained within your development directory.",
                ],
            ));
        }

        let relative_path = dir.strip_prefix(&dev_dir).wrap_system_err(
            "We were unable to determine the repository's fully qualified name.",
            &["Make sure that you are currently within a repository contained within your development directory."]
        )?;

        let svc = relative_path.components().next().ok_or_user_err(
            "Current directory is not a valid repository.",
            &["Make sure that you are currently within a repository contained within your development directory."],
        )?;

        let svc_name = svc.as_os_str().to_string_lossy().to_string();
        repo_from_svc_and_path(
            &self.config,
            Some(svc_name),
            relative_path.strip_prefix(svc).unwrap_or(relative_path),
            false,
        )
    }
}

fn get_service_and_path_from_str(repo: &str) -> (Option<String>, std::path::PathBuf) {
    match repo.split_once(':') {
        Some((svc, path)) => (Some(svc.to_string()), path.into()),
        None => (None, repo.into()),
    }
}

fn repo_from_str(
    config: &Config,
    repo: &str,
    fallback_to_default: bool,
) -> Result<Repo, human_errors::Error> {
    let (svc, path) = get_service_and_path_from_str(repo);
    repo_from_svc_and_path(config, svc, &path, fallback_to_default)
}

fn repo_from_svc_and_path(
    config: &Config,
    svc: Option<String>,
    path: &std::path::Path,
    fallback_to_default: bool,
) -> Result<Repo, human_errors::Error> {
    let svc = match svc {
        Some(svc) => config.get_service(&svc),
        None if fallback_to_default => config.get_default_service().ok_or_user_err(
            "No services configured for use with Git Tool.",
            &["Make sure that you have registered a service in your Git-Tool config using `git-tool config add services/NAME`."]),
        None => Err(human_errors::user(
            format!("The path '{}' used to resolve a repo did not start with a service namespace.", path.display()),
            &["Make sure that your repository starts with the name of a service, such as 'gh:sierrasoftworks/git-tool'."],
        )),
    }?;

    let name_length = svc.pattern.split_terminator('/').count();
    let name_parts: Vec<String> = path
        .components()
        .take(name_length)
        .map(|c| c.as_os_str().to_str().unwrap().to_string())
        .collect();

    let true_path = std::path::PathBuf::from(&svc.name).join(path);

    if name_parts.len() != name_length {
        Err(human_errors::user(
            format!(
                "The service '{}' requires a repository name in the form '{}', but you provided '{}'.",
                &svc.name,
                &svc.pattern,
                path.display()
            ),
            &[
                "Make sure that you are using a repository name which matches the service's expected pattern.",
            ],
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
    use super::*;
    use crate::engine::Target;
    use crate::test::get_dev_dir;
    use std::sync::Arc;

    fn resolver() -> TrueResolver {
        TrueResolver::new(Arc::new(Config::for_dev_directory(&get_dev_dir())))
    }

    fn core() -> Core {
        Core::builder()
            .with_config(Config::for_dev_directory(&get_dev_dir()))
            .build()
    }

    #[test]
    fn resolves_a_repo_from_an_identifier() {
        let core = core();
        let identifier: Identifier = "gh:sierrasoftworks/test1".parse().unwrap();

        let repo: Repo = core.resolve(&identifier).unwrap();
        assert_eq!(repo.get_full_name(), "sierrasoftworks/test1");
        assert_eq!(
            repo.get_path(),
            get_dev_dir()
                .join("gh")
                .join("sierrasoftworks")
                .join("test1")
        );
    }

    #[test]
    fn resolves_a_repo_from_a_name() {
        let core = core();
        let repo: Repo = core.resolve("gh:sierrasoftworks/test1").unwrap();
        assert_eq!(repo.get_full_name(), "sierrasoftworks/test1");
    }

    #[test]
    fn resolves_the_best_matching_repo() {
        let resolver = resolver();
        let repo: Repo = resolver.resolve(&"gh:spt1".parse().unwrap()).unwrap();
        assert_eq!(repo.get_full_name(), "spartan563/test1");
    }

    #[test]
    fn resolves_a_new_repo_which_does_not_exist_yet() {
        let resolver = resolver();
        let repo: Repo = resolver
            .resolve(&"gh:sierrasoftworks/test3".parse().unwrap())
            .unwrap();
        assert_eq!(repo.get_full_name(), "sierrasoftworks/test3");
        assert_eq!(
            repo.get_path(),
            get_dev_dir()
                .join("gh")
                .join("sierrasoftworks")
                .join("test3")
        );
    }

    #[test]
    fn resolves_against_the_default_service() {
        let resolver = resolver();
        let repo: Repo = resolver
            .resolve(&"sierrasoftworks/test3".parse().unwrap())
            .unwrap();
        assert_eq!(&repo.service, "gh");
        assert_eq!(repo.get_full_name(), "sierrasoftworks/test3");
    }

    #[test]
    fn resolves_exact_match_when_multiple_repos_match() {
        let resolver = resolver();
        let repo: Repo = resolver
            .resolve(&"sierrasoftworks/test1".parse().unwrap())
            .unwrap();
        assert_eq!(&repo.service, "gh");
        assert_eq!(repo.get_full_name(), "sierrasoftworks/test1");
    }

    #[test]
    fn resolves_a_repo_from_an_absolute_path() {
        let resolver = resolver();

        let repo = resolver
            .repo_from_path(
                &get_dev_dir()
                    .join("gh")
                    .join("sierrasoftworks")
                    .join("test1"),
            )
            .unwrap();
        assert_eq!(repo.get_full_name(), "sierrasoftworks/test1");
    }

    #[test]
    fn resolves_the_current_repo_via_the_mock() {
        let core = Core::builder()
            .with_default_config()
            .with_mock_resolver(|mock| {
                mock.expect_get_current_repo()
                    .returning(|| Ok(Repo::new("gh:test/current", "/dev/gh/test/current".into())));
            })
            .build();

        let repo: Repo = core.resolve(()).unwrap();
        assert_eq!(repo.get_full_name(), "test/current");
    }

    #[test]
    fn enumerates_all_repos() {
        let core = core();
        let repos: Vec<Repo> = core.resolve_many(()).unwrap();
        assert_eq!(repos.len(), 7);

        let example = repos
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
    fn enumerates_repos_for_a_service() {
        let core = core();
        let service = core.config().get_service("gh").unwrap();

        let repos: Vec<Repo> = core.resolve_many(service).unwrap();
        assert_eq!(repos.len(), 5);
    }
}
