use std::path::{Path, PathBuf};

/// Converts a path into an appropriate native path by handling the
/// conversion of `/` into path segments.
///
/// This method is useful for converting paths which are provided
/// as strings or other formats into a native path format. It's
/// primarily useful on Windows where the path separator is `\`
/// but `/` is commonly used in other contexts.
///
/// ## Example
/// ```
/// use git_tool::fs::to_native_path;
///
/// to_native_path("a/b/c");
/// ```
pub fn to_native_path<T: Into<PathBuf>>(path: T) -> PathBuf {
    let mut output = PathBuf::new();
    let input: PathBuf = path.into();

    output.extend(input.components().flat_map(|c| {
        match c {
            std::path::Component::Normal(n) => n
                .to_str()
                .unwrap()
                .split('/')
                .map(|p| std::path::Component::Normal(p.as_ref()))
                .collect(),
            _ => vec![c],
        }
    }));

    output
}

/// Resolves a path pattern (which may include non-wildcard segments) into a list of matching
/// directories.
///
/// This method is useful for resolving a path pattern which may include wildcards such as
/// 'myorg/*/myrepo' into a list of directories which match that pattern. It represents a
/// capability enhancement over the functionality provided by [`get_child_directories`].
pub fn resolve_directories(
    from: &Path,
    pattern: &str,
) -> Result<Vec<PathBuf>, human_errors::Error> {
    if !from.exists() {
        Ok(Vec::new())
    } else if let Some((first_segment, rest)) = pattern.split_once('/') {
        if first_segment == "*" {
            Ok(get_child_directories(from)?
                .map(|dir| resolve_directories(&dir, rest))
                .collect::<Result<Vec<Vec<PathBuf>>, human_errors::Error>>()?
                .into_iter()
                .flatten()
                .collect())
        } else {
            resolve_directories(&from.join(first_segment), rest)
        }
    } else if pattern == "*" {
        get_child_directories(from).map(|dirs| dirs.collect())
    } else {
        Ok(vec![from.join(pattern)])
    }
}

pub fn get_child_directories(
    from: &Path,
) -> Result<impl Iterator<Item = PathBuf>, human_errors::Error> {
    Ok(from.read_dir().map_err(|e| match e.kind() {
        std::io::ErrorKind::NotFound => human_errors::user(
            format!("The path '{}' does not exist.", from.display()),
            &["Please check that the path is correct and that you have permission to access it."],
        ),
        std::io::ErrorKind::NotADirectory => human_errors::user(
            format!("The path '{}' is not a directory.", from.display()),
            &["Please check that this path is a directory and that you have not accidentally created a file here instead."],
        ),
        _ => human_errors::wrap_system(
            e,
            format!("Could not enumerate directories in '{}' due to an OS-level error.", from.display()),
            &["Check that Git-Tool has permission to read this directory."],
        ),
    })?.filter_map(|entry| {
        if let Ok(entry) = entry {
            if entry.file_type().is_ok_and(|ft| ft.is_dir()) {
                Some(entry.path())
            } else {
                None
            }
        } else {
            None
        }
    }))
}

/// Creates a filesystem link at `link` which points to `original`.
///
/// On Unix-like systems this creates a symbolic link. On Windows it creates a
/// directory junction (which does not require elevated privileges or developer
/// mode), and so is only suitable for linking directories on that platform.
///
/// The `original` path is expected to exist; if it does not, the link is not
/// created and an error is returned.
#[tracing::instrument(err)]
pub fn create_link(original: &Path, link: &Path) -> Result<(), human_errors::Error> {
    if !original.exists() {
        return Err(human_errors::user(
            format!(
                "Could not create a link to '{}' because it does not exist.",
                original.display()
            ),
            &["Make sure that the path you are linking to exists before trying again."],
        ));
    }

    if let Some(parent) = link.parent()
        && !parent.exists()
    {
        std::fs::create_dir_all(parent).map_err(|e| {
            human_errors::wrap_system(
                e,
                format!(
                    "Could not create the parent directory '{}' for the link due to an OS-level error.",
                    parent.display()
                ),
                &["Check that Git-Tool has permission to write to this directory."],
            )
        })?;
    }

    create_link_native(original, link).map_err(|e| {
        human_errors::wrap_system(
            e,
            format!(
                "Could not create a link at '{}' pointing to '{}' due to an OS-level error.",
                link.display(),
                original.display()
            ),
            &["Check that Git-Tool has permission to create links in this location."],
        )
    })
}

#[cfg(unix)]
fn create_link_native(original: &Path, link: &Path) -> std::io::Result<()> {
    std::os::unix::fs::symlink(original, link)
}

#[cfg(windows)]
fn create_link_native(original: &Path, link: &Path) -> std::io::Result<()> {
    if original.is_dir() {
        junction::create(original, link)
    } else {
        std::os::windows::fs::symlink_file(original, link)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::get_dev_dir;
    use itertools::Itertools;
    use rstest::rstest;

    #[test]
    fn test_to_native_path() {
        assert_eq!(
            to_native_path("a/b/c"),
            PathBuf::from("a").join("b").join("c")
        );

        assert_eq!(
            to_native_path(get_dev_dir().join("github.com/sierrasoftworks")),
            get_dev_dir().join("github.com").join("sierrasoftworks")
        );
    }

    #[test]
    fn get_child_directories() {
        let children = resolve_directories(&get_dev_dir().join("gh"), "*/*")
            .expect("to get child directories");

        assert_eq!(children.len(), 5);

        assert!(children.iter().any(|p| {
            p == &get_dev_dir()
                .join("gh")
                .join("sierrasoftworks")
                .join("test1")
        }));
        assert!(children.iter().any(|p| {
            p == &get_dev_dir()
                .join("gh")
                .join("sierrasoftworks")
                .join("test2")
        }));
        assert!(
            children
                .iter()
                .any(|p| p == &get_dev_dir().join("gh").join("spartan563").join("test1"))
        );
        assert!(
            children
                .iter()
                .any(|p| p == &get_dev_dir().join("gh").join("spartan563").join("test2"))
        );

        assert!(
            resolve_directories(
                &get_dev_dir()
                    .join("gh")
                    .join("sierrasoftworks")
                    .join("test1")
                    .join(".gitkeep"),
                "*"
            )
            .is_err()
        );
    }

    #[rstest]
    #[case("gh/sierrasoftworks/*", &["gh/sierrasoftworks/test1", "gh/sierrasoftworks/test12", "gh/sierrasoftworks/test2"])]
    #[case("gh/*/test1", &["gh/sierrasoftworks/test1", "gh/spartan563/test1"])]
    fn test_resolve_directories(#[case] pattern: &str, #[case] expected: &[&str]) {
        let directories = resolve_directories(&get_dev_dir(), pattern).expect("to get directories");

        let prefix = get_dev_dir();
        let result = directories
            .iter()
            .map(|p| {
                assert!(p.starts_with(&prefix));
                assert!(p.is_dir());
                p.components()
                    .skip(prefix.components().count())
                    .map(|c| c.as_os_str().to_string_lossy())
                    .join("/")
            })
            .sorted()
            .collect::<Vec<_>>();
        let expected = expected
            .iter()
            .map(|s| s.to_string())
            .sorted()
            .collect::<Vec<_>>();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_create_link_directory() {
        let temp = tempfile::tempdir().unwrap();
        let original = temp.path().join("original");
        std::fs::create_dir_all(&original).unwrap();
        std::fs::write(original.join("marker.txt"), "hello").unwrap();

        let link = temp.path().join("linked");
        create_link(&original, &link).expect("the link should be created");

        assert!(link.exists());
        assert_eq!(
            std::fs::read_to_string(link.join("marker.txt")).unwrap(),
            "hello"
        );
    }

    #[test]
    fn test_create_link_missing_original() {
        let temp = tempfile::tempdir().unwrap();
        let original = temp.path().join("does-not-exist");
        let link = temp.path().join("linked");

        assert!(create_link(&original, &link).is_err());
    }
}
