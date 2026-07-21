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

/// Records the observed state (hashed from the provided fields) with IJON so
/// that AFL++ treats reaching a previously-unseen state as new coverage. This
/// helps the fuzzer explore stateful code paths — such as sequences of git
/// branch operations — whose interesting behaviour is invisible to plain edge
/// coverage. On non-fuzzing builds it is a no-op.
pub fn record_state(fields: &[String]) {
    #[cfg(fuzzing)]
    {
        let mut sorted: Vec<&String> = fields.iter().collect();
        sorted.sort();

        // FNV-style fold of the sorted field set into a single IJON state value.
        let mut hash: u32 = 0;
        for field in sorted {
            for byte in field.bytes() {
                hash = hash.wrapping_mul(31).wrapping_add(byte as u32);
            }
            hash = hash.wrapping_mul(31).wrapping_add(1);
        }

        afl::ijon_set!(hash);
    }

    #[cfg(not(fuzzing))]
    let _ = fields;
}
