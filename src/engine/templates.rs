use super::{Config, Repo, Service, Target};
use gotmpl::{MissingKey, Template, TemplateError, Value};
use percent_encoding::{AsciiSet, CONTROLS, utf8_percent_encode};
use serde::Serialize;
use tracing_batteries::prelude::*;

const URL_QUERY_ENCODE: &AsciiSet = &CONTROLS
    .add(b' ')
    .add(b'"')
    .add(b'<')
    .add(b'>')
    .add(b'#')
    .add(b'`')
    .add(b'?')
    .add(b'{')
    .add(b'}');

pub fn render(tmpl: &str, context: Value) -> Result<String, human_errors::Error> {
    debug!("Rendering template '{}' with context {}", tmpl, context);
    Template::new("git-tool")
        .missing_key(MissingKey::Error)
        .func("urlquery", urlquery)
        .parse(tmpl)
        .and_then(|template| template.execute_to_string(&context))
        .map_err(|e| human_errors::wrap_user(
            e.to_string(),
            format!("We couldn't render your template '{tmpl}'."),
            &["Check that your template follows the Go template syntax here: https://golang.org/pkg/text/template/"],
        ))
}

fn urlquery(args: &[Value]) -> gotmpl::Result<Value> {
    match args {
        [Value::String(value)] => Ok(Value::String(
            utf8_percent_encode(value, URL_QUERY_ENCODE)
                .to_string()
                .into(),
        )),
        [_] => Err(TemplateError::TypeMismatch {
            expected: "string",
            got: "non-string",
        }),
        _ => Err(TemplateError::Exec(format!(
            "urlquery requires exactly 1 argument, got {}",
            args.len()
        ))),
    }
}

#[tracing::instrument(err, skip(context, items))]
pub fn render_list<S: AsRef<str>>(
    items: Vec<S>,
    context: Value,
) -> Result<Vec<String>, human_errors::Error> {
    let mut out = Vec::with_capacity(items.len());

    for item in items {
        let rendered = render(item.as_ref(), context.clone())?;
        out.push(rendered);
    }

    Ok(out)
}

pub fn repo_context<'a>(config: &'a Config, repo: &'a Repo) -> Result<Value, human_errors::Error> {
    match config.get_service(&repo.service) {
        Ok(service) => repo_template_context(repo, Some(service)),
        Err(_) => repo_template_context(repo, None),
    }
}

pub fn repo_context_without_service(repo: &Repo) -> Result<Value, human_errors::Error> {
    repo_template_context(repo, None)
}

pub fn target_context(target: &(impl Target + ?Sized)) -> Result<Value, human_errors::Error> {
    serialize_context(&TemplateContext {
        target: TargetTemplateContext::new(target),
        repo: None,
        service: None,
    })
}

fn repo_template_context(
    repo: &Repo,
    service: Option<&Service>,
) -> Result<Value, human_errors::Error> {
    let service_context = ServiceTemplateContext::new(repo, service);
    let website = service.and_then(|service| service.get_website(repo).ok());
    let git_url = service.and_then(|service| service.get_git_url(repo).ok());

    serialize_context(&TemplateContext {
        target: TargetTemplateContext {
            name: repo.get_full_name(),
            path: path_string(repo),
            exists: repo.exists(),
        },
        repo: Some(RepoTemplateContext {
            full_name: repo.get_full_name(),
            name: repo.get_name(),
            namespace: &repo.namespace,
            domain: &repo.service,
            exists: repo.exists(),
            valid: repo.valid(),
            path: path_string(repo),
            website,
            http_url: git_url.clone(),
            git_url,
            service: service_context,
        }),
        service: Some(service_context),
    })
}

fn serialize_context(context: &TemplateContext<'_>) -> Result<Value, human_errors::Error> {
    gotmpl::to_value(context).map_err(|error| {
        human_errors::wrap_user(
            error.to_string(),
            "We couldn't prepare the template context.",
            &["Check that template context fields contain supported values."],
        )
    })
}

fn path_string(target: &(impl Target + ?Sized)) -> String {
    target.get_path().to_str().unwrap_or_default().to_string()
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct TemplateContext<'a> {
    target: TargetTemplateContext,
    repo: Option<RepoTemplateContext<'a>>,
    service: Option<ServiceTemplateContext<'a>>,
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct TargetTemplateContext {
    name: String,
    path: String,
    exists: bool,
}

impl TargetTemplateContext {
    fn new(target: &(impl Target + ?Sized)) -> Self {
        Self {
            name: target.get_name(),
            path: path_string(target),
            exists: target.exists(),
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct RepoTemplateContext<'a> {
    full_name: String,
    name: String,
    namespace: &'a str,
    domain: &'a str,
    exists: bool,
    valid: bool,
    path: String,
    website: Option<String>,
    #[serde(rename = "GitURL")]
    git_url: Option<String>,
    #[serde(rename = "HttpURL")]
    http_url: Option<String>,
    service: ServiceTemplateContext<'a>,
}

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "PascalCase")]
struct ServiceTemplateContext<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<&'a str>,
    domain: &'a str,
    directory_glob: Option<&'a str>,
    pattern: Option<&'a str>,
}

impl<'a> ServiceTemplateContext<'a> {
    fn new(repo: &'a Repo, service: Option<&'a Service>) -> Self {
        Self {
            name: service.map(|service| service.name.as_str()),
            domain: service.map_or(repo.service.as_str(), |service| service.name.as_str()),
            directory_glob: service.map(|service| service.pattern.as_str()),
            pattern: service.map(|service| service.pattern.as_str()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::Scratchpad;
    use gotmpl::tmap;
    use std::path::PathBuf;

    #[test]
    fn render_basic_repo() -> Result<(), Box<dyn std::error::Error>> {
        let cfg = Config::default();
        let repo = Repo::new(
            "ghp:sierrasoftworks/git-tool",
            PathBuf::from("/test/ghp/sierrasoftworks/git-tool"),
        );

        let context = repo_context(&cfg, &repo)?;

        assert_eq!(
            render("{{ .Repo.Name }}", context.clone()).unwrap(),
            "git-tool"
        );
        assert_eq!(
            render("{{ .Repo.FullName }}", context.clone()).unwrap(),
            "sierrasoftworks/git-tool"
        );
        assert_eq!(
            render("{{ .Repo.Namespace }}", context.clone()).unwrap(),
            "sierrasoftworks"
        );
        assert_eq!(
            render("{{ .Repo.Domain }}", context.clone()).unwrap(),
            "ghp"
        );
        assert_eq!(
            render("{{ .Repo.Path }}", context.clone()).unwrap(),
            "/test/ghp/sierrasoftworks/git-tool"
        );
        assert_eq!(
            render("{{ .Repo.Website }}", context.clone()).unwrap(),
            "https://github.com/sierrasoftworks/git-tool"
        );
        assert_eq!(
            render("{{ .Repo.GitURL }}", context.clone()).unwrap(),
            "https://github.com/sierrasoftworks/git-tool.git"
        );
        assert_eq!(
            render("{{ .Repo.HttpURL }}", context.clone()).unwrap(),
            "https://github.com/sierrasoftworks/git-tool.git"
        );

        assert_eq!(
            render("{{ .Target.Name }}", context.clone()).unwrap(),
            "sierrasoftworks/git-tool"
        );
        assert_eq!(
            render("{{ .Target.Path }}", context.clone()).unwrap(),
            "/test/ghp/sierrasoftworks/git-tool"
        );

        assert_eq!(
            render("{{ .Service.Domain }}", context.clone()).unwrap(),
            "ghp"
        );
        assert_eq!(
            render("{{ .Repo.Service.Domain }}", context).unwrap(),
            "ghp"
        );

        Ok(())
    }

    #[test]
    fn render_basic_scratchpad() -> Result<(), Box<dyn std::error::Error>> {
        let scratch = Scratchpad::new("2020w07", PathBuf::from("/test/scratch/2020w07"));
        let context = target_context(&scratch)?;

        assert_eq!(
            render("{{ .Target.Name }}", context.clone()).unwrap(),
            "2020w07"
        );
        assert_eq!(
            render("{{ .Target.Path }}", context).unwrap(),
            "/test/scratch/2020w07"
        );

        Ok(())
    }

    #[test]
    fn render_advanced_repo() -> Result<(), Box<dyn std::error::Error>> {
        let repo = Repo::new(
            "gh:sierrasoftworks/git-tool",
            PathBuf::from("/test/github.com/sierrasoftworks/git-tool"),
        );

        assert_eq!(render("{{ with .Repo }}{{ .Service.Domain }}:{{ .FullName }}{{ else }}{{ .Target.Name }}{{ end }}", repo_context_without_service(&repo)?).unwrap(), "gh:sierrasoftworks/git-tool");

        Ok(())
    }

    #[test]
    fn render_advanced_scratchpad() -> Result<(), Box<dyn std::error::Error>> {
        let scratch = Scratchpad::new("2020w07", PathBuf::from("/test/scratch/2020w07"));

        assert_eq!(render("{{ with .Repo }}{{ .Service.Domain }}:{{ .FullName }}{{ else }}{{ .Target.Name }}{{ end }}", target_context(&scratch)?).unwrap(), "2020w07");

        Ok(())
    }

    #[test]
    fn render_invalid_syntax() -> Result<(), Box<dyn std::error::Error>> {
        let scratch = Scratchpad::new("2020w07", PathBuf::from("/test/scratch/2020w07"));

        render("{{ .Target.Name", target_context(&scratch)?).unwrap_err();

        Ok(())
    }

    #[test]
    fn render_unicode_literal_text() {
        let context = tmap! {
            "Name" => Value::String("世界".into())
        };

        assert_eq!(
            render("Héllo, {{ .Name }}", context).unwrap(),
            "Héllo, 世界"
        );
    }

    #[test]
    fn render_rejects_unicode_inside_action_without_panicking() {
        let fuzz_input = [
            123, 123, 32, 119, 105, 116, 104, 32, 46, 255, 255, 255, 255, 255, 255, 255, 1, 123,
            32, 46, 32, 125, 125, 61, 123, 123, 32, 36, 46, 86, 97, 108, 117, 117, 101, 32, 125,
            125, 123, 123, 32, 101, 108, 115, 101, 32, 125, 125, 101, 109, 152, 139, 134, 132, 123,
            32, 101, 110, 100, 32, 125, 125,
        ];
        let template = String::from_utf8_lossy(&fuzz_input);

        assert!(render(&template, Value::Nil).is_err());
    }

    #[test]
    fn render_rejects_control_characters_inside_action_without_hanging() {
        let fuzz_input = [
            123, 123, 32, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 119, 105, 116, 104, 32, 47, 42,
            97, 109, 101, 32, 125, 125, 123, 123, 32, 46, 32, 125, 125, 61, 123, 123, 32, 36, 46,
            86, 97, 108, 117, 101, 32, 125, 125, 123, 123, 32, 101, 108, 115, 101, 32, 125, 125,
            101, 109, 112, 116, 121, 123, 123, 32, 101, 110, 100, 32, 125, 125,
        ];
        let template = String::from_utf8(fuzz_input.to_vec()).unwrap();

        assert!(render(&template, Value::Nil).is_err());
    }

    #[test]
    fn render_rejects_unterminated_action_without_hanging() {
        for template in ["{{ ", "{{ .", "{{ if ", "{{ range ", "{{ \"a", "{{ /*"] {
            assert!(
                render(template, Value::Nil).is_err(),
                "expected unterminated action '{template}' to be rejected"
            );
        }
    }

    #[test]
    fn render_rejects_unterminated_action_from_fuzz_input_without_hanging() {
        // Mirrors how the `templates` fuzz target extracts the template from the
        // NUL-delimited corpus entry, which reduces to an unterminated `{{ ` action.
        let fuzz_input = [
            123, 123, 32, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 119, 105, 116, 104, 32, 47, 42,
            97, 109, 101, 32, 125, 125, 123, 123, 32, 46, 32, 125, 125, 61, 123, 123, 32, 36, 46,
            86, 97, 108, 117, 101, 32, 125, 125, 123, 123, 32, 101, 108, 115, 101, 32, 125, 125,
            101, 109, 112, 116, 121, 123, 123, 32, 101, 110, 100, 32, 125, 125,
        ];
        let template = fuzz_input
            .split(|byte| *byte == 0)
            .next()
            .map(|field| String::from_utf8_lossy(field).into_owned())
            .unwrap_or_default();

        assert_eq!(template, "{{ ");
        assert!(render(&template, Value::Nil).is_err());
    }

    #[test]
    fn render_trims_whitespace_after_unicode_without_panicking() {
        let context = tmap! {
            "Name" => Value::String("world".into())
        };

        for (template, expected) in [
            ("é {{- .Name }}", "éworld"),
            ("⸀ {{- .Name }}", "⸀world"),
            ("héllo{{- .Name }}", "hélloworld"),
            ("héllo {{- .Name }}", "hélloworld"),
        ] {
            assert_eq!(render(template, context.clone()).unwrap(), expected);
        }
    }

    #[test]
    fn render_urlquery_preserves_legacy_path_encoding() -> Result<(), Box<dyn std::error::Error>> {
        let context = gotmpl::to_value("sierrasoftworks/example/git tool/世界")?;

        assert_eq!(
            render("{{ . | urlquery }}", context).unwrap(),
            "sierrasoftworks/example/git%20tool/%E4%B8%96%E7%95%8C"
        );

        Ok(())
    }

    #[test]
    fn serde_context_preserves_nil_fields() -> Result<(), Box<dyn std::error::Error>> {
        let repo = Repo::new("gh:sierrasoftworks/git-tool", PathBuf::from("/test"));
        let context = repo_context_without_service(&repo)?;

        assert_eq!(
            render("{{ .Repo.Website }}", context.clone()).unwrap(),
            "<no value>"
        );
        assert_eq!(
            render("{{ .Service.Pattern }}", context).unwrap(),
            "<no value>"
        );

        Ok(())
    }

    #[test]
    fn render_invalid_field() -> Result<(), Box<dyn std::error::Error>> {
        let scratch = Scratchpad::new("2020w07", PathBuf::from("/test/scratch/2020w07"));

        render("{{ .Target.UnknownField }}", target_context(&scratch)?).unwrap_err();

        Ok(())
    }
}
