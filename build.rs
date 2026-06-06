fn main() {
    #[cfg(windows)]
    embed_windows_resources();
}

#[cfg(windows)]
fn embed_windows_resources() {
    let mut res = winresource::WindowsResource::new();

    // Re-use the logo shipped with the documentation site as the executable icon.
    res.set_icon("docs/.vuepress/public/favicon.ico");

    // The assembly identity requires a four-part `major.minor.build.revision`
    // version, so pad the three-part Cargo version with a trailing `.0`.
    let manifest = std::fs::read_to_string("assets/git-tool.exe.manifest")
        .expect("failed to read Windows manifest template");
    let version = std::env::var("CARGO_PKG_VERSION").expect("CARGO_PKG_VERSION not set by Cargo");
    let manifest = manifest.replace("{VERSION}", &format!("{version}.0"));
    res.set_manifest(&manifest);

    res.set("ProductName", "Git-Tool");
    res.set(
        "FileDescription",
        "A productivity tool for managing your git repositories.",
    );
    res.set("CompanyName", "Sierra Softworks");
    res.set("LegalCopyright", "Copyright © Sierra Softworks");

    res.compile()
        .expect("failed to embed Windows executable resources");

    println!("cargo:rerun-if-changed=assets/git-tool.exe.manifest");
    println!("cargo:rerun-if-changed=docs/.vuepress/public/favicon.ico");
}
