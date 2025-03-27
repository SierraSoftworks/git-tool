use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use tracing_batteries::prelude::debug;

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
/// use crate::fs::to_native_path;
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

/// Gets the list of child directories which match a given pattern such as "*/*".
///
/// This method enumerates directories which match a representative glob pattern
/// consisting of wildcards separated by slashes. Unlike a full glob, this internally
/// is treated as a depth marker (with "*/*" corresponding to a depth of 2).
///
/// Internally, this method dispatches to [`get_directory_tree_to_depth`].
///
/// ## Example
/// ```
/// use crate::fs::get_child_directories;
///
/// get_child_directories("/".into(), "*");
/// ```
pub fn get_child_directories(
    from: &Path,
    pattern: &str,
) -> Result<Vec<PathBuf>, crate::errors::Error> {
    let depth = pattern.split('/').count();

    get_directory_tree_to_depth(from, depth)
}

/// Gets the list of child directories which appear at a given depth relative to a root path.
///
/// This method recursively enumerates child directories, returning the list of directory paths
/// which appear a given depth below a provided root path.
pub fn get_directory_tree_to_depth(
    from: &Path,
    depth: usize,
) -> Result<Vec<PathBuf>, crate::errors::Error> {
    if depth == 0 {
        return Ok(vec![from.to_owned()]);
    }

    debug!(
        "Enumerating child directories of '{}' to depth {}",
        from.display(),
        depth
    );

    let mut directories = Vec::new();

    match from.read_dir() {
        Ok(dirs) => {
            for dir in dirs.flatten() {
                if let Ok(ft) = dir.file_type() {
                    if ft.is_dir() {
                        let children = get_directory_tree_to_depth(&dir.path(), depth - 1)?;
                        directories.extend(children);
                    }
                }
            }
        }
        Err(e) if e.kind() == ErrorKind::NotFound => {}
        Err(e) => {
            return Err(crate::errors::user_with_internal(
                &format!(
                    "Could not enumerate directories in '{}' due to an OS-level error.",
                    from.display()
                ),
                "Check that Git-Tool has permission to read this directory.",
                e,
            ));
        }
    }

    Ok(directories)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::get_dev_dir;

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
        let children = super::get_child_directories(&get_dev_dir().join("gh"), "*/*")
            .expect("to get child directories");

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
}
