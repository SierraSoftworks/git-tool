#![no_main]

use git_tool::commands;
use libfuzzer_sys::fuzz_target;
use std::sync::LazyLock;

mod common;

fuzz_target!(|data: &[u8]| {
    static APP: LazyLock<clap::Command> = LazyLock::new(commands::app);

    let mut arguments = vec!["git-tool".to_string()];
    arguments.extend(common::fields(data));

    let _ = APP.clone().try_get_matches_from(arguments);
});
