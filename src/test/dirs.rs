pub fn get_repo_root() -> std::path::PathBuf {
    std::path::PathBuf::from(file!())
        .parent()
        .and_then(|f| f.parent())
        .and_then(|f| f.parent())
        .unwrap()
        .to_path_buf()
}

pub fn get_dev_dir() -> std::path::PathBuf {
    get_repo_root().join("test").join("devdir")
}