use super::Resolver;
use crate::engine::{Branch, Core};

/// Resolves a branch from its raw name.
impl Resolver<&str, Branch> for Core {
    fn resolve(&self, source: &str) -> Result<Branch, human_errors::Error> {
        source.parse()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_a_branch() {
        let core = Core::builder().with_default_config().build();
        let branch: Branch = core.resolve("feature/test").unwrap();
        assert_eq!(branch.as_str(), "feature/test");
    }
}
