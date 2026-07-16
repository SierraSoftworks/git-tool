#![no_main]

use git_tool::engine::{render, render_list};
use gtmpl::Value;
use libfuzzer_sys::fuzz_target;
use std::collections::HashMap;

mod common;

fuzz_target!(|data: &[u8]| {
    let mut fields = data.splitn(3, |byte| *byte == 0);
    let template = common::bounded_text(fields.next().unwrap_or_default());
    let name = common::bounded_text(fields.next().unwrap_or_default());
    let value = common::bounded_text(fields.next().unwrap_or_default());
    let context = Value::Object(HashMap::from([
        ("Name".to_string(), Value::String(name)),
        ("Value".to_string(), Value::String(value)),
    ]));

    if let Ok(rendered) = render(&template, context.clone()) {
        assert_eq!(render(&template, context.clone()).unwrap(), rendered);
        assert_eq!(
            render_list(vec![template, "literal".to_string()], context).unwrap(),
            vec![rendered, "literal".to_string()]
        );
    }
});
