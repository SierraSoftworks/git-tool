use std::path::PathBuf;

pub fn to_native_path<T: Into<PathBuf>>(path: T) -> std::path::PathBuf {
    let mut output = std::path::PathBuf::new();
    let input: PathBuf = path.into();

    output.extend(input.components().flat_map(|c| {
        match c {
            std::path::Component::Normal(n) => n
                .to_str()
                .unwrap()
                .split("/")
                .map(|p| std::path::Component::Normal(p.as_ref()))
                .collect(),
            _ => vec![c],
        }
    }));

    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::get_dev_dir;
    use std::path::PathBuf;

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
}
