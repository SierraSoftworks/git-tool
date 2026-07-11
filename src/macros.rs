#[cfg(not(debug_assertions))]
#[macro_export]
macro_rules! version {
    () => {
        env!("CARGO_PKG_VERSION")
    };
    ($prefix:expr) => {
        format!("{}{}", $prefix, env!("CARGO_PKG_VERSION"))
    };
}

#[cfg(debug_assertions)]
#[macro_export]
macro_rules! version {
    () => {
        "0.0.0-dev"
    };
    ($prefix:expr) => {
        format!("{}0.0.0-dev", $prefix)
    };
}

/// Like [`eprintln!`], but never panics when the standard error stream has been
/// torn down (for example when the terminal is closed while a command is still
/// running).
///
/// The built-in `eprintln!`/`println!` macros unwrap the underlying write and
/// panic on failure (`failed printing to stderr: ...`). When the terminal or a
/// downstream pipe goes away mid-write that turns an unremarkable shutdown into
/// a crash which our panic hook then reports as an exception. There is nowhere
/// left to surface the message in that situation, so we simply drop it.
macro_rules! safe_eprintln {
    ($($arg:tt)*) => {{
        use std::io::Write;
        let _ = writeln!(std::io::stderr(), $($arg)*);
    }};
}
