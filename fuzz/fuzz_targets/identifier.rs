use afl::fuzz;
use git_tool::engine::Identifier;

mod common;

fn main() {
    fuzz!(|data: &[u8]| {
        let mut fields = data.splitn(2, |byte| *byte == 0);
        let source = common::bounded_text(fields.next().unwrap_or_default());
        let partial = common::bounded_text(fields.next().unwrap_or_default());

        if let Ok(identifier) = source.parse::<Identifier>() {
            assert!(!identifier.path.trim().is_empty());

            let rendered = identifier.to_string();
            let reparsed = rendered
                .parse::<Identifier>()
                .expect("a rendered identifier to remain valid");
            assert_eq!(reparsed, identifier);

            if let Ok(resolved) = identifier.resolve(&partial) {
                let reparsed = resolved
                    .to_string()
                    .parse::<Identifier>()
                    .expect("a resolved identifier to remain valid");
                assert_eq!(reparsed, resolved);
            }
        }
    });
}
