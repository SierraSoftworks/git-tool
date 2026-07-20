use super::{Config, Repo, Scratchpad, Service, Target, target::TempTarget};
use gotmpl::{MissingKey, Template, Value};
use tracing_batteries::prelude::*;

macro_rules! map(
    { $($key:expr => $value:expr),+ } => {
        {
            let mut m = ::std::collections::BTreeMap::new();
            $(
                m.insert(String::from($key).into(), $value);
            )+
            m.into()
        }
     };
);

pub fn render(tmpl: &str, context: Value) -> Result<String, human_errors::Error> {
    debug!("Rendering template '{}' with context {}", tmpl, context);
    Template::new("git-tool")
        .missing_key(MissingKey::Error)
        .parse(tmpl)
        .and_then(|template| template.execute_to_string(&context))
        .map_err(|e| human_errors::wrap_user(
            e.to_string(),
            format!("We couldn't render your template '{tmpl}'."),
            &["Check that your template follows the Go template syntax here: https://golang.org/pkg/text/template/"],
        ))
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

pub fn repo_context<'a>(config: &'a Config, repo: &'a Repo) -> Value {
    match config.get_service(&repo.service) {
        Ok(service) => RepoWithService { repo, service }.into(),
        Err(_) => repo.into(),
    }
}

struct RepoWithService<'a> {
    repo: &'a Repo,
    service: &'a Service,
}

#[allow(clippy::from_over_into)]
impl Into<Value> for RepoWithService<'_> {
    fn into(self) -> Value {
        let service: Value = self.service.into();

        Value::Map(map! {
            "Target" => Value::Map(map!{
                "Name" => Value::String(self.repo.get_full_name().into()),
                "Path" => Value::String(String::from(self.repo.get_path().to_str().unwrap_or_default()).into()),
                "Exists" => Value::Bool(self.repo.exists())
            }),
            "Repo" => Value::Map(map!{
                "FullName" => Value::String(self.repo.get_full_name().into()),
                "Name" => Value::String(self.repo.get_name().into()),
                "Namespace" => Value::String(self.repo.namespace.clone().into()),
                "Domain" => Value::String(self.repo.service.clone().into()),
                "Exists" => Value::Bool(self.repo.exists()),
                "Valid" => Value::Bool(self.repo.valid()),
                "Path" => Value::String(String::from(self.repo.get_path().to_str().unwrap_or_default()).into()),
                "Website" => Value::String(self.service.get_website(self.repo).unwrap_or_default().into()),
                "GitURL" => Value::String(self.service.get_git_url(self.repo).unwrap_or_default().into()),
                "HttpURL" => Value::String(self.service.get_git_url(self.repo).unwrap_or_default().into()),
                "Service" => service.clone()
            }),
            "Service" => service
        })
    }
}

#[allow(clippy::from_over_into)]
impl Into<Value> for &Service {
    fn into(self) -> Value {
        Value::Map(map! {
            "Name" => Value::String(self.name.clone().into()),
            "Domain" => Value::String(self.name.clone().into()),
            "DirectoryGlob" => Value::String(self.pattern.clone().into()),
            "Pattern" => Value::String(self.pattern.clone().into())
        })
    }
}

#[allow(clippy::from_over_into)]
impl Into<Value> for &Repo {
    fn into(self) -> Value {
        let service = Value::Map(map! {
            "Domain" => Value::String(self.service.clone().into()),
            "DirectoryGlob" => Value::Nil,
            "Pattern" => Value::Nil
        });

        Value::Map(map! {
            "Target" => Value::Map(map!{
                "Name" => Value::String(self.get_full_name().into()),
                "Path" => Value::String(String::from(self.get_path().to_str().unwrap_or_default()).into()),
                "Exists" => Value::Bool(self.exists())
            }),
            "Repo" => Value::Map(map!{
                "FullName" => Value::String(self.get_full_name().into()),
                "Name" => Value::String(self.get_name().into()),
                "Namespace" => Value::String(self.namespace.clone().into()),
                "Domain" => Value::String(self.service.clone().into()),
                "Exists" => Value::Bool(self.exists()),
                "Valid" => Value::Bool(self.valid()),
                "Path" => Value::String(String::from(self.get_path().to_str().unwrap_or_default()).into()),
                "Website" => Value::Nil,
                "GitURL" => Value::Nil,
                "HttpURL" => Value::Nil,
                "Service" => service.clone()
            }),
            "Service" => service
        })
    }
}

#[allow(clippy::from_over_into)]
impl Into<Value> for &Scratchpad {
    fn into(self) -> Value {
        Value::Map(map! {
            "Target" => Value::Map(map!{
                "Name" => Value::String(self.get_name().into()),
                "Path" => Value::String(String::from(self.get_path().to_str().unwrap_or_default()).into()),
                "Exists" => Value::Bool(self.exists())
            }),
            "Repo" => Value::Nil,
            "Service" => Value::Nil
        })
    }
}

#[allow(clippy::from_over_into)]
impl Into<Value> for &TempTarget {
    fn into(self) -> Value {
        Value::Map(map! {
            "Target" => Value::Map(map!{
                "Name" => Value::String(self.get_name().into()),
                "Path" => Value::String(String::from(self.get_path().to_str().unwrap_or_default()).into()),
                "Exists" => Value::Bool(self.exists())
            }),
            "Repo" => Value::Nil,
            "Service" => Value::Nil
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn render_basic_repo() {
        let cfg = Config::default();
        let repo = Repo::new(
            "ghp:sierrasoftworks/git-tool",
            PathBuf::from("/test/ghp/sierrasoftworks/git-tool"),
        );

        let context = repo_context(&cfg, &repo);

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
    }

    #[test]
    fn render_basic_scratchpad() {
        let scratch = Scratchpad::new("2020w07", PathBuf::from("/test/scratch/2020w07"));

        assert_eq!(
            render("{{ .Target.Name }}", (&scratch).into()).unwrap(),
            "2020w07"
        );
        assert_eq!(
            render("{{ .Target.Path }}", (&scratch).into()).unwrap(),
            "/test/scratch/2020w07"
        );
    }

    #[test]
    fn render_advanced_repo() {
        let repo = Repo::new(
            "gh:sierrasoftworks/git-tool",
            PathBuf::from("/test/github.com/sierrasoftworks/git-tool"),
        );

        assert_eq!(render("{{ with .Repo }}{{ .Service.Domain }}:{{ .FullName }}{{ else }}{{ .Target.Name }}{{ end }}", (&repo).into()).unwrap(), "gh:sierrasoftworks/git-tool");
    }

    #[test]
    fn render_advanced_scratchpad() {
        let scratch = Scratchpad::new("2020w07", PathBuf::from("/test/scratch/2020w07"));

        assert_eq!(render("{{ with .Repo }}{{ .Service.Domain }}:{{ .FullName }}{{ else }}{{ .Target.Name }}{{ end }}", (&scratch).into()).unwrap(), "2020w07");
    }

    #[test]
    fn render_invalid_syntax() {
        let scratch = Scratchpad::new("2020w07", PathBuf::from("/test/scratch/2020w07"));

        render("{{ .Target.Name", (&scratch).into()).unwrap_err();
    }

    #[test]
    fn render_unicode_literal_text() {
        let context = Value::Map(map! {
            "Name" => Value::String("世界".into())
        });

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
        let context = Value::Map(map! {
            "Name" => Value::String("world".into())
        });

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
    fn render_invalid_field() {
        let scratch = Scratchpad::new("2020w07", PathBuf::from("/test/scratch/2020w07"));

        render("{{ .Target.UnknownField }}", (&scratch).into()).unwrap_err();
    }
}
