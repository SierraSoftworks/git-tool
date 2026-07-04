use super::{ResolveMany, Resolver};
use crate::engine::{Identifier, Repo, Scratchpad, Service, TempMode, TempTarget};

// The mock's expectation methods keep the familiar `get_*` names so that tests
// read naturally (`mock.expect_get_best_repo()...`); the [`Resolver`]
// implementations below map each strongly-typed resolution onto the matching
// expectation.
mockall::mock! {
    pub Resolver {
        pub fn get_temp(&self, mode: TempMode) -> Result<TempTarget, human_errors::Error>;

        pub fn get_scratchpads(&self) -> Result<Vec<Scratchpad>, human_errors::Error>;
        pub fn get_scratchpad(&self, name: &str) -> Result<Scratchpad, human_errors::Error>;
        pub fn get_current_scratchpad(&self) -> Result<Scratchpad, human_errors::Error>;

        pub fn get_repos(&self) -> Result<Vec<Repo>, human_errors::Error>;
        pub fn get_repos_for(&self, service: &Service) -> Result<Vec<Repo>, human_errors::Error>;
        pub fn get_best_repo(&self, identifier: &Identifier) -> Result<Repo, human_errors::Error>;
        pub fn get_current_repo(&self) -> Result<Repo, human_errors::Error>;
    }
}

impl Resolver<(), Repo> for MockResolver {
    fn resolve(&self, _source: ()) -> Result<Repo, human_errors::Error> {
        self.get_current_repo()
    }
}

impl Resolver<&Identifier, Repo> for MockResolver {
    fn resolve(&self, source: &Identifier) -> Result<Repo, human_errors::Error> {
        self.get_best_repo(source)
    }
}

impl ResolveMany<(), Repo> for MockResolver {
    fn resolve_many(&self, _source: ()) -> Result<Vec<Repo>, human_errors::Error> {
        self.get_repos()
    }
}

impl ResolveMany<&Service, Repo> for MockResolver {
    fn resolve_many(&self, source: &Service) -> Result<Vec<Repo>, human_errors::Error> {
        self.get_repos_for(source)
    }
}

impl Resolver<(), Scratchpad> for MockResolver {
    fn resolve(&self, _source: ()) -> Result<Scratchpad, human_errors::Error> {
        self.get_current_scratchpad()
    }
}

impl Resolver<&str, Scratchpad> for MockResolver {
    fn resolve(&self, source: &str) -> Result<Scratchpad, human_errors::Error> {
        self.get_scratchpad(source)
    }
}

impl ResolveMany<(), Scratchpad> for MockResolver {
    fn resolve_many(&self, _source: ()) -> Result<Vec<Scratchpad>, human_errors::Error> {
        self.get_scratchpads()
    }
}

impl Resolver<TempMode, TempTarget> for MockResolver {
    fn resolve(&self, source: TempMode) -> Result<TempTarget, human_errors::Error> {
        self.get_temp(source)
    }
}
