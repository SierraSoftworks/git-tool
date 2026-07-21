use afl::fuzz;
use git_tool::commands;
use std::sync::LazyLock;

mod common;

fn main() {
    fuzz!(|data: &[u8]| {
        static APP: LazyLock<clap::Command> = LazyLock::new(commands::app);

        let mut arguments = vec!["git-tool".to_string()];
        arguments.extend(common::fields(data));

        let _ = APP.clone().try_get_matches_from(arguments);
    });
}
