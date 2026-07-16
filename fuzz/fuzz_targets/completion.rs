#![no_main]

use git_tool::{
    completion,
    engine::{Branch, Core},
};
use libfuzzer_sys::fuzz_target;
use std::sync::OnceLock;

mod common;

fn core() -> &'static Core {
    static CORE: OnceLock<Core> = OnceLock::new();
    CORE.get_or_init(|| Core::builder().with_default_config().build())
}

fuzz_target!(|data: &[u8]| {
    let (context_flag, input) = data.split_first().unwrap_or((&0, &[]));
    let tokens = common::fields(input);
    let arguments = std::iter::once("completion-fuzz").chain(tokens.iter().map(String::as_str));
    let matches = clap::Command::new("completion-fuzz")
        .arg(
            clap::Arg::new("args")
                .action(clap::ArgAction::Append)
                .allow_hyphen_values(true),
        )
        .try_get_matches_from(arguments);

    let Ok(matches) = matches else {
        return;
    };

    let context = (*context_flag & 1 != 0)
        .then(|| "current".parse::<Branch>().expect("a valid context branch"));

    if let Ok(parsed) = completion::parse::<Branch>(core(), context.as_ref(), &matches) {
        assert!(!parsed.target.as_str().is_empty());
        for (key, value) in &parsed.env {
            let original = format!("{key}={value}");
            assert!(tokens.contains(&original));
        }

        parsed
            .launch_app(core())
            .expect("successfully parsed arguments to produce a launchable app");
    }
});
