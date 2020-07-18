#[macro_export]
macro_rules! version {
    () => {
        env!("CARGO_PKG_VERSION")
    };
    ($prefix:expr) => {
        format!("{}{}", $prefix, env!("CARGO_PKG_VERSION"))
    };
}
