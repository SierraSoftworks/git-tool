use crate::errors;
use std::fmt::Display;
use std::str::FromStr;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Identifier {
    pub scope: String,
    pub path: String,
}

#[allow(dead_code)]
impl Identifier {
    pub fn namespace(&self) -> &str {
        self.path
            .rsplit_once('/')
            .map_or("", |(namespace, _)| namespace)
    }

    pub fn name(&self) -> &str {
        self.path
            .rsplit_once('/')
            .map_or(self.path.as_str(), |(_, name)| name)
    }

    pub fn path_segments(&self) -> impl Iterator<Item = &str> {
        self.path.split('/').filter(|segment| !segment.is_empty())
    }

    pub fn resolve(&self, partial: &str) -> Result<Self, errors::Error> {
        if partial.trim().is_empty() {
            return Err(human_errors::user(
                &format!(
                    "Could not resolve a new repository identifier based on '{}' when the target is empty.",
                    self
                ),
                "Make sure that you specify a valid target repository path such as 'namespace/name'",
            ));
        }

        if partial.contains(':') {
            return partial.parse();
        }

        let mut old_segments = self.path_segments().collect::<Vec<_>>();

        let n = old_segments.len();
        partial
            .rsplit('/')
            .enumerate()
            .take(n)
            .for_each(|(idx, segment)| {
                old_segments[n - idx - 1] = segment;
            });

        Ok(Identifier {
            scope: self.scope.clone(),
            path: old_segments.join("/").to_string(),
        })
    }
}

impl Display for Identifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match (self.scope.as_str(), self.path.as_str()) {
            ("", path) => write!(f, "{}", path),
            (scope, path) => write!(f, "{}:{}", scope, path),
        }
    }
}

impl FromStr for Identifier {
    type Err = errors::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.trim().is_empty() {
            return Err(human_errors::user(
                &format!(
                    "The repository identifier '{s}' was not in a valid format and could not be understood."
                ),
                "Make sure you specify a valid repository identifier in the form 'service:namespace/name' or 'namespace/name'",
            ));
        }

        let mut id = Identifier {
            scope: String::new(),
            path: String::new(),
        };

        let mut s = s.trim();
        if let Some((scope, rest)) = s.split_once(':') {
            s = rest;
            id.scope = scope.to_string();
        }

        id.path = s.to_string();
        Ok(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("gh:SierraSoftworks/git-tool", "gh", "SierraSoftworks/git-tool")]
    #[case("ado:myorg/myteam/myrepo", "ado", "myorg/myteam/myrepo")]
    #[case("SierraSoftworks/git-tool", "", "SierraSoftworks/git-tool")]
    fn test_parse(#[case] source: &str, #[case] expected_scope: &str, #[case] expected_path: &str) {
        let id: Identifier = source.parse().expect("id to be valid");

        assert_eq!(id.scope, expected_scope);
        assert_eq!(id.path, expected_path);
    }

    #[rstest]
    #[case("gh:SierraSoftworks/git-tool", "bender", "gh:SierraSoftworks/bender")]
    #[case(
        "gh:SierraSoftworks/git-tool",
        "notheotherben/cv",
        "gh:notheotherben/cv"
    )]
    #[case(
        "gh:SierraSoftworks/git-tool",
        "ado:myorg/myteam/myrepo",
        "ado:myorg/myteam/myrepo"
    )]
    #[case(
        "ado:myorg/myteam/myrepo",
        "gh:SierraSoftworks/git-tool",
        "gh:SierraSoftworks/git-tool"
    )]
    fn test_resolve(#[case] source: &str, #[case] relative: &str, #[case] expected: &str) {
        let id: Identifier = source.parse().expect("id to be valid");
        let new = id.resolve(relative).expect("new to be valid");
        assert_eq!(format!("{new}"), expected);
    }

    #[rstest]
    #[case("gh:SierraSoftworks/git-tool", &["SierraSoftworks", "git-tool"])]
    #[case("ado:myorg/myteam/myrepo", &["myorg", "myteam", "myrepo"])]
    fn test_path_segments(#[case] source: &str, #[case] expected: &[&str]) {
        let id: Identifier = source.parse().expect("id to be valid");
        let segments = id.path_segments().collect::<Vec<_>>();
        assert_eq!(segments, expected);
    }
}
