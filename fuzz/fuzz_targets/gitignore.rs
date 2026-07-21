use afl::fuzz;
use git_tool::online::gitignore::parse_managed_section_fuzz;

mod common;

fn main() {
    fuzz!(|data: &[u8]| {
        let input = String::from_utf8_lossy(common::bounded_bytes(data)).into_owned();

        // Parsing arbitrary .gitignore content must never panic or loop, and must
        // be deterministic.
        let parsed = parse_managed_section_fuzz(&input);
        assert_eq!(parsed, parse_managed_section_fuzz(&input));

        // Re-parsing the normalized rendering must also terminate cleanly.
        if let Some(rendered) = parsed {
            let _ = parse_managed_section_fuzz(&rendered);
        }
    });
}
