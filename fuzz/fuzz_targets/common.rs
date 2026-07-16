#![allow(dead_code)]

use std::sync::OnceLock;

const MAX_OPERATIONS: usize = 8;
const MAX_INPUT_LENGTH: usize = 16 * 1024;
const MAX_TEXT_LENGTH: usize = 128;

pub fn bounded_bytes(data: &[u8]) -> &[u8] {
    &data[..data.len().min(MAX_INPUT_LENGTH)]
}

pub fn bounded_text(data: &[u8]) -> String {
    String::from_utf8_lossy(&data[..data.len().min(MAX_TEXT_LENGTH)]).into_owned()
}

pub fn fields(data: &[u8]) -> Vec<String> {
    data.split(|byte| *byte == 0)
        .take(MAX_OPERATIONS)
        .map(bounded_text)
        .collect()
}

pub fn operations(data: &[u8]) -> Vec<(u8, String)> {
    data.split(|byte| *byte == 0)
        .filter(|field| !field.is_empty())
        .take(MAX_OPERATIONS)
        .map(|field| (field[0], bounded_text(&field[1..])))
        .collect()
}

pub fn runtime() -> &'static tokio::runtime::Runtime {
    static RUNTIME: OnceLock<tokio::runtime::Runtime> = OnceLock::new();

    RUNTIME.get_or_init(|| {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("the fuzz runtime to initialize")
    })
}
