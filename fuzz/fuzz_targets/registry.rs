use afl::fuzz;
use git_tool::online::registry::Entry;

mod common;

fn main() {
    fuzz!(|data: &[u8]| {
        let input = common::bounded_bytes(data);

        // Mirrors how the file-backed and GitHub registries deserialize untrusted
        // registry entries (`serde_yaml::from_slice`/`from_str::<Entry>`).
        if let Ok(entry) = serde_yaml::from_slice::<Entry>(input) {
            let canonical =
                serde_yaml::to_string(&entry).expect("a parsed registry entry to serialize");
            let reparsed: Entry =
                serde_yaml::from_str(&canonical).expect("a serialized registry entry to parse");
            let reserialized =
                serde_yaml::to_string(&reparsed).expect("a reparsed registry entry to serialize");

            let canonical_value: serde_yaml::Value = serde_yaml::from_str(&canonical)
                .expect("a serialized registry entry to be valid YAML");
            let reserialized_value: serde_yaml::Value = serde_yaml::from_str(&reserialized)
                .expect("a reserialized registry entry to be valid YAML");

            assert_eq!(canonical_value, reserialized_value);
        }
    });
}
