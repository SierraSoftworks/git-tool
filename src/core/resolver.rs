use super::{Config, Scratchpad, Error, Service, Repo, errors};
use std::env;
use glob::glob;
use chrono::prelude::*;
use crate::search;

pub trait Resolver {
    fn get_scratchpads(&self) -> Result<Vec<Scratchpad>, Error>;
    fn get_scratchpad(&self, name: &str) -> Result<Scratchpad, Error>;
    fn get_current_scratchpad(&self) -> Result<Scratchpad, Error>;

    fn get_current_repo(&self) -> Result<Repo, Error>;
    fn get_repo(&self, path: &std::path::PathBuf) -> Result<Repo, Error>;
    fn get_repos(&self) -> Result<Vec<Repo>, Error>;
    fn get_repos_for(&self, svc: &Service) -> Result<Vec<Repo>, Error>;

    fn get_best_repo(&self, name: &str) -> Result<Repo, Error>;
}

pub struct FileSystemResolver {
    config: Config,
}

impl FileSystemResolver {
    pub fn new(config: Config) -> Self{
        Self {
            config
        }
    }
}

impl Resolver for FileSystemResolver {
    fn get_scratchpads(&self) -> Result<Vec<Scratchpad>, Error> {
        let dirs = self.config.get_scratch_directory().read_dir()?;

        let mut scratchpads = vec![];
        for dir in dirs {
            let dir_info = dir?;
            if dir_info.file_type()?.is_dir() {
                if let Some(name) = dir_info.file_name().to_str() {
                    scratchpads.push(Scratchpad::new(
                        name, 
                        dir_info.path().to_path_buf()));
                }
            }
        }

        Ok(scratchpads)
    }

    fn get_scratchpad(&self, name: &str) -> Result<Scratchpad, Error> {
        Ok(Scratchpad::new(
            name.clone(), 
            self.config.get_scratch_directory().join(name.clone())))
    }

    fn get_current_scratchpad(&self) -> Result<Scratchpad, Error> {
        let time = Local::now();

        self.get_scratchpad(time.format("%Yw%V").to_string().as_str())
    }

    fn get_current_repo(&self) -> Result<Repo, Error> {
        let cwd = env::current_dir()?;

        match self.get_repo(&cwd) {
            Ok(repo) => Ok(repo),
            Err(e) => Err(errors::user_with_internal(
                format!("Current directory ('{}') is not a valid repository.", cwd.display()).as_str(),
                format!("Make sure that you are currently within a repository contained within your development directory ('{}').", self.config.get_dev_directory().canonicalize()?.display()).as_str(),
                e))
        }
    }

    fn get_repo(&self, path: &std::path::PathBuf) -> Result<Repo, Error> {
        let dev_dir = self.config.get_dev_directory().canonicalize()?;
        let dir = if path.is_absolute() { path.canonicalize()? } else { self.config.get_dev_directory().join(path).canonicalize()? };

        if !dir.starts_with(&dev_dir) || dir == dev_dir {
            return Err(errors::user(
                "Current directory is not a valid repository.",
                format!("Make sure that you are currently within a repository contained within your development directory ('{}').", dev_dir.display()).as_str()))
        }

        match dir.strip_prefix(&dev_dir) {
            Ok(relative_path) => repo_from_relative_path(&self.config, &relative_path.to_path_buf()),
            Err(e) => Err(errors::system_with_internal(
                "We were unable to determine the repository's fully qualified name.", 
                format!("Make sure that you are currently within a repository contained within your development directory ('{}').", dev_dir.display()).as_str(),
                e.to_string()))
        }
    }

    fn get_best_repo(&self, name: &str) -> Result<Repo, Error> {
        let true_name = self.config.get_alias(name).unwrap_or(name.to_string());

        match self.get_repo(&std::path::PathBuf::from(true_name)) {
            Ok(repo) => return Ok(repo),
            Err(_) => {}
        }

        let all_repos = self.get_repos()?;
        let repos: Vec<&Repo> = all_repos.iter().filter(|r| search::matches(r.get_full_name().as_str(), name)).collect();

        match repos.len() {
            0 => Err(errors::user("No matching repository found.", "Please check that you have provided the correct name for the repository and try again.")),
            1 => Ok((*repos.first().unwrap()).clone()),
            _ => Err(errors::user("The repository name you provided matched more than one repository.", "Try entering a repository name that is unique, or the fully qualified repository name, to avoid confusion."))
        }
    }

    fn get_repos(&self) -> Result<Vec<Repo>, Error> {
        let mut repos = vec![];

        for svc_dir in self.config.get_dev_directory().read_dir()? {
            match svc_dir {
                Ok(dir) => {
                    if dir.file_type()?.is_dir() {
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

    fn get_repos_for(&self, svc: &Service) -> Result<Vec<Repo>, Error> {
        let path = self.config.get_dev_directory().join(svc.get_domain());

        let mut repos = vec![];

        match glob(path.join(svc.get_pattern()).to_str().unwrap()) {
            Ok(entries) => {
                for entry in entries  {
                    match entry {
                        Ok(path) => {
                            if path.is_dir() {
                                match self.get_repo(&path) {
                                    Ok(repo) => repos.push(repo),
                                    Err(e) => { return Err(e) }
                                }
                            }
                        },
                        Err(_) => {}
                    }
                }
            },
            Err(e) => {
                return Err(errors::user_with_internal(
                    format!("The glob pattern used for the '{}' service was invalid.", svc.get_domain()).as_str(),
                    "Please ensure that the glob pattern you have used for this service (in your config file) is valid and try again.",
                    e))
            }
        }

        Ok(repos)
    }
}

fn service_from_relative_path<'a>(config: &'a Config, relative_path: &std::path::PathBuf) -> Result<&'a Service, Error> {
    if !relative_path.is_relative() {
        return Err(errors::system(
            format!("The path '{}' used to resolve a repo was not relative.", relative_path.display()).as_str(),
            "Please report this issue to us on GitHub, including the command you ran, so that we can troubleshoot the problem."))
    }
    
    let mut components = relative_path.components();
    match components.next() {
        Some(std::path::Component::Normal(name)) => {
            match config.get_service(name.to_str().unwrap()) {
                Some(svc) => Ok(svc),
                None => Err(errors::user(
                        format!("The service '{}' was not recognized as a repository provider.", name.to_str().unwrap()).as_str(),
                        format!("Make sure that you have registered the service in your git-tool config using `git-tool config add services/{}`", name.to_str().unwrap()).as_str()))
            }
        },
        _ => Err(errors::user(
            format!("The path '{}' used to resolve a repo did not start with a service domain name.", relative_path.display()).as_str(),
            "Make sure that your repository starts with the name of a service, such as 'github.com/sierrasoftworks/git-tool'."))
    }
    
}

fn repo_from_relative_path<'a>(config: &'a Config, relative_path: &std::path::PathBuf) -> Result<Repo, Error> {
    if !relative_path.is_relative() {
        return Err(errors::system(
            format!("The path '{}' used to resolve a repo was not relative.", relative_path.display()).as_str(),
            "Please report this issue to us on GitHub, including the command you ran, so that we can troubleshoot the problem."))
    }
    
    let svc = service_from_relative_path(config, relative_path)?;

    let name_length = svc.get_pattern().split_terminator("/").count() + 1;

    let name_parts: Vec<String> = relative_path.components().take(name_length).map(|c| c.as_os_str().to_str().unwrap().to_string()).collect();

    if name_parts.len() != name_length {
        Err(errors::user(
            format!("The service '{}' requires a repository name in the form '{}', but we got '{}'.", svc.get_domain(), svc.get_pattern(), relative_path.display()).as_str(),
            "Make sure that your repository is correctly named for the service you are using."))
    } else {
        Ok(Repo::new(name_parts.join("/").as_str(), config.get_dev_directory().join(relative_path)))
    }
}

#[cfg(test)]
pub struct MockResolver {
    repo: Option<Repo>,
    repos: Vec<Repo>,
    scratchpads: Vec<Scratchpad>,
    current_date: DateTime<Local>,
    error: Option<Error>
}

#[cfg(test)]
impl MockResolver {
    pub fn set_repo(&mut self, repo: Repo) {
        self.repo = Some(repo)
    }

    pub fn set_repos(&mut self, repos: Vec<Repo>) {
        self.repos = repos
    }

    pub fn set_scratchpads(&mut self, scratchpads: Vec<Scratchpad>) {
        self.scratchpads = scratchpads
    }
}

#[cfg(test)]
impl Default for MockResolver {
    fn default() -> Self {
        Self {
            repo: None,
            repos: Vec::new(),
            scratchpads: Vec::new(),
            current_date: Local.ymd(2020, 01, 02).and_hms(03, 04, 05),
            error: None
        }
    }
}

#[cfg(test)]
impl Resolver for MockResolver {
    fn get_scratchpads(&self) -> Result<Vec<Scratchpad>, Error> {
        match self.error.clone() {
            Some(err) => Err(err),
            None => Ok(self.scratchpads.clone())
        }
    }
    fn get_scratchpad(&self, name: &str) -> Result<Scratchpad, Error> {
        Ok(Scratchpad::new(
            name.clone(), 
            std::path::PathBuf::from("/dev/scratch").join(name.clone())))
    }

    fn get_current_scratchpad(&self) -> Result<Scratchpad, Error> {
        self.get_scratchpad(self.current_date.format("%Yw%V").to_string().as_str())
    }

    fn get_current_repo(&self) -> Result<Repo, Error> {
        match self.repo.clone() {
            Some(repo) => Ok(repo),
            None => Err(errors::user(
                "Current directory is not a valid repository.",
                "Make sure that you are currently within a repository contained within your development directory."))
        }
    }

    fn get_repo(&self, _path: &std::path::PathBuf) -> Result<Repo, Error> {
        match self.repo.clone() {
            Some(repo) => Ok(repo),
            None => Err(errors::user(
                "Current directory is not a valid repository.",
                "Make sure that you are currently within a repository contained within your development directory."))
        }
    }

    fn get_repos(&self) -> Result<Vec<Repo>, Error> {
        match self.error.clone() {
            Some(err) => Err(err),
            None => Ok(self.repos.clone())
        }
    }

    fn get_repos_for(&self, svc: &Service) -> Result<Vec<Repo>, Error> {
        match self.error.clone() {
            Some(err) => Err(err),
            None => Ok(self.repos.iter().filter(|r| r.get_domain() == svc.get_domain()).map(|r| r.clone()).collect())
        }
    }
    fn get_best_repo(&self, name: &str) -> Result<Repo, Error> {
        let path = std::path::PathBuf::from(name);
        self.get_repo(&path)
    }
}

#[cfg(test)]
mod tests {

}