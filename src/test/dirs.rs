pub fn get_repo_root() -> std::path::PathBuf {
    std::path::PathBuf::from(file!())
        .canonicalize()
        .unwrap()
        .parent()
        .and_then(|f| f.parent())
        .and_then(|f| f.parent())
        .unwrap()
        .to_path_buf()
}

pub fn get_dev_dir() -> std::path::PathBuf {
    get_repo_root().join("test").join("devdir")
}

#[test]
fn test_dev_dir() {
    assert!(get_dev_dir().exists());
    assert!(get_dev_dir().join("gh").exists());
}
