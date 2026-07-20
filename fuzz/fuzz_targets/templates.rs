#![no_main]

use git_tool::engine::{render, render_list};
use gotmpl::tmap;
use libfuzzer_sys::fuzz_target;

mod common;

fuzz_target!(|data: &[u8]| {
    let mut fields = data.splitn(3, |byte| *byte == 0);
    let template = common::bounded_text(fields.next().unwrap_or_default());
    let name = common::bounded_text(fields.next().unwrap_or_default());
    let value = common::bounded_text(fields.next().unwrap_or_default());
    let selector = data.first().copied().unwrap_or_default() as usize;
    let literal = template.replace(['{', '}'], "_");
    let context = tmap! {
        "Name" => name.clone(),
        "Value" => value.clone(),
        "Enabled" => selector.is_multiple_of(2),
        "Count" => selector as i64,
        "Empty" => gotmpl::Value::Nil,
        "Items" => vec![name, value.clone()],
        "Nested" => tmap! {
            "Value" => value,
        },
    };

    if let Ok(rendered) = render(&template, context.clone()) {
        assert_eq!(render(&template, context.clone()).unwrap(), rendered);
        assert_eq!(
            render_list(vec![template, "literal".to_string()], context.clone()).unwrap(),
            vec![rendered, "literal".to_string()]
        );
    }

    let structured_templates = [
        "{{ $root := . }}{{ if .Enabled }}{{ $root.Name }}{{ else }}{{ .Nested.Value }}{{ end }}",
        "{{ range $index, $item := .Items }}{{ $index }}={{ $item }};{{ else }}empty{{ end }}",
        "{{ with .Nested }}{{ .Value | printf \"%q\" }}{{ else }}empty{{ end }}",
        "{{/* braces {{ }} and Unicode 世界 inside a comment */}}{{ len .Items }}:{{ index .Items 0 }}",
        "{{ if and .Enabled (ne .Count 0) }}enabled{{ else if .Empty }}nil{{ else }}disabled{{ end }}",
        "{{ $value := .Nested.Value }}{{ printf \"%s/%d\" $value .Count }}",
        "{{ .Empty.Field }}",
        "{{ .Name.Field }}",
    ];
    let structured_template = structured_templates[selector % structured_templates.len()];
    let structured_result = render(structured_template, context.clone());
    if let Ok(rendered) = structured_result {
        assert_eq!(
            render(structured_template, context.clone()).unwrap(),
            rendered
        );
        assert_eq!(
            render_list(vec![structured_template], context.clone()).unwrap(),
            vec![rendered]
        );
    }

    let trim_template = format!("{literal} \t\r\n{{{{- .Value -}}}}\n\t {literal}");
    assert_eq!(
        render(&trim_template, context).unwrap(),
        format!("{}{value}{}", literal.trim_end(), literal.trim_start())
    );
});
