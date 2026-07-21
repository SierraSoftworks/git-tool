use afl::fuzz;
use git_tool::engine::RepoConfig;

mod common;

fn main() {
    fuzz!(|data: &[u8]| {
        let input = common::bounded_bytes(data);

        if let Ok(config) = RepoConfig::from_bytes(input) {
            let canonical = config
                .to_yaml()
                .expect("a parsed repository configuration to serialize");
            let hash = config
                .hash()
                .expect("a parsed repository configuration to hash");
            let reparsed = RepoConfig::from_bytes(canonical.as_bytes())
                .expect("a serialized repository configuration to parse");

            assert_eq!(canonical, reparsed.to_yaml().unwrap());
            assert_eq!(hash, reparsed.hash().unwrap());
            assert_eq!(hash.len(), 64);
            assert!(hash.bytes().all(|byte| byte.is_ascii_hexdigit()));
        }
    });
}
