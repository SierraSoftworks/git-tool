#[cfg(test)]
use mocktopus::{macros::*, mocking::*};

#[cfg_attr(test, mockable)]
pub fn prompt(msg: &str, default: &str) -> String {
    match write!(super::output::output(), "{}", msg) {
        Ok(_) => (),
        Err(_) => {
            return default.into();
        }
    };

    let mut input = String::new();
    match std::io::stdin().read_line(&mut input) {
        Ok(_) => (),
        Err(_) => {
            return default.into();
        }
    };

    input.trim().to_string()
}

#[cfg(test)]
pub fn mock(answer: Option<&str>) {
    let answer = answer.map(|s| s.to_string());
    prompt.mock_safe(move |_prompt, default| {
        MockResult::Return(answer.clone().unwrap_or(default.into()))
    });
}
