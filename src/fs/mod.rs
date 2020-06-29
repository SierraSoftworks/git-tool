
pub fn to_native_path(path: std::path::PathBuf) -> std::path::PathBuf {
    let mut output = std::path::PathBuf::new();
    output.extend(path.components().flat_map(|c| match c {
        std::path::Component::Normal(n) => n.to_str().unwrap().split("/").map(|p| std::path::Component::Normal(p.as_ref())).collect(),
        _ => vec![c]
    }));

    output
}