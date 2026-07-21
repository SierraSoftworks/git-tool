use afl::fuzz;
use git_tool::engine::Config;
use std::io::Cursor;

mod common;

fn main() {
    fuzz!(|data: &[u8]| {
        let input = common::bounded_bytes(data);

        if let Ok(config) = Config::from_reader(Cursor::new(input)) {
            let canonical = config
                .to_string()
                .expect("a parsed configuration to serialize");
            let reparsed = Config::from_reader(Cursor::new(canonical.as_bytes()))
                .expect("a serialized configuration to parse");
            let reserialized = reparsed
                .to_string()
                .expect("a reparsed configuration to serialize");
            let canonical_value: serde_yaml::Value = serde_yaml::from_str(&canonical)
                .expect("a serialized configuration to be valid YAML");
            let reserialized_value: serde_yaml::Value = serde_yaml::from_str(&reserialized)
                .expect("a reserialized configuration to be valid YAML");

            assert_eq!(canonical_value, reserialized_value);
        }
    });
}
