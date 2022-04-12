use super::{errors, Config, Repo, Scratchpad, Service, Target};
use gtmpl::{template, Value};

macro_rules! map(
    { $($key:expr => $value:expr),+ } => {
        {
            let mut m = ::std::collections::HashMap::new();
            $(
                m.insert(String::from($key), $value);
            )+
            m
        }
     };
);

pub fn render(tmpl: &str, context: Value) -> Result<String, errors::Error> {
    template(tmpl, context).map_err(|e| errors::user_with_internal(
        format!("We couldn't render your template '{}'.", tmpl).as_str(),
        "Check that your template follows the Go template syntax here: https://golang.org/pkg/text/template/",
        errors::detailed_message(&e.to_string())))
}

pub fn render_list<S: AsRef<str>>(
    items: Vec<S>,
    context: Value,
) -> Result<Vec<String>, errors::Error> {
    let mut out = Vec::new();
    out.reserve(items.len());

    for item in items {
        let rendered = render(item.as_ref(), context.clone())?;
        out.push(rendered);
    }

    Ok(out)
}

pub fn repo_context<'a>(config: &'a Config, repo: &'a Repo) -> Value {
    match config.get_service(&repo.service) {
        Some(service) => RepoWithService { repo, service }.into(),
        None => repo.into(),
    }
}

struct RepoWithService<'a> {
    repo: &'a Repo,
    service: &'a Service,
}

impl<'a> std::convert::Into<Value> for RepoWithService<'a> {
    fn into(self) -> Value {
        let service: Value = self.service.into();

        Value::Object(map! {
            "Target" => Value::Object(map!{
                "Name" => Value::String(self.repo.get_full_name()),
                "Path" => Value::String(String::from(self.repo.get_path().to_str().unwrap_or_default())),
                "Exists" => Value::Bool(self.repo.exists())
            }),
            "Repo" => Value::Object(map!{
                "FullName" => Value::String(self.repo.get_full_name()),
                "Name" => Value::String(self.repo.get_name()),
                "Namespace" => Value::String(self.repo.namespace.clone()),
                "Domain" => Value::String(self.repo.service.clone()),
                "Exists" => Value::Bool(self.repo.exists()),
                "Valid" => Value::Bool(self.repo.valid()),
                "Path" => Value::String(String::from(self.repo.get_path().to_str().unwrap_or_default())),
                "Website" => Value::String(self.service.get_website(self.repo).unwrap_or_default()),
                "GitURL" => Value::String(self.service.get_git_url(self.repo).unwrap_or_default()),
                "HttpURL" => Value::String(self.service.get_git_url(self.repo).unwrap_or_default()),
                "Service" => service.clone()
            }),
            "Service" => service.clone()
        })
    }
}

impl<'a> std::convert::Into<Value> for &Service {
    fn into(self) -> Value {
        Value::Object(map! {
            "Name" => Value::String(self.name.clone()),
            "Domain" => Value::String(self.name.clone()),
            "DirectoryGlob" => Value::String(self.pattern.clone()),
            "Pattern" => Value::String(self.pattern.clone())
        })
    }
}

impl<'a> std::convert::Into<Value> for &Repo {
    fn into(self) -> Value {
        let service = Value::Object(map! {
            "Domain" => Value::String(self.service.clone()),
            "DirectoryGlob" => Value::NoValue,
            "Pattern" => Value::NoValue
        });

        Value::Object(map! {
            "Target" => Value::Object(map!{
                "Name" => Value::String(self.get_full_name()),
                "Path" => Value::String(String::from(self.get_path().to_str().unwrap_or_default())),
                "Exists" => Value::Bool(self.exists())
            }),
            "Repo" => Value::Object(map!{
                "FullName" => Value::String(self.get_full_name()),
                "Name" => Value::String(self.get_name()),
                "Namespace" => Value::String(self.namespace.clone()),
                "Domain" => Value::String(self.service.clone()),
                "Exists" => Value::Bool(self.exists()),
                "Valid" => Value::Bool(self.valid()),
                "Path" => Value::String(String::from(self.get_path().to_str().unwrap_or_default())),
                "Website" => Value::NoValue,
                "GitURL" => Value::NoValue,
                "HttpURL" => Value::NoValue,
                "Service" => service.clone()
            }),
            "Service" => service.clone()
        })
    }
}

impl std::convert::Into<Value> for &Scratchpad {
    fn into(self) -> Value {
        Value::Object(map! {
            "Target" => Value::Object(map!{
                "Name" => Value::String(self.get_name()),
                "Path" => Value::String(String::from(self.get_path().to_str().unwrap_or_default())),
                "Exists" => Value::Bool(self.exists())
            }),
            "Repo" => Value::NoValue,
            "Service" => Value::NoValue
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
            "gh:sierrasoftworks/git-tool",
            PathBuf::from("/test/github.com/sierrasoftworks/git-tool"),
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
        assert_eq!(render("{{ .Repo.Domain }}", context.clone()).unwrap(), "gh");
        assert_eq!(
            render("{{ .Repo.Path }}", context.clone()).unwrap(),
            "/test/github.com/sierrasoftworks/git-tool"
        );
        assert_eq!(
            render("{{ .Repo.Website }}", context.clone()).unwrap(),
            "https://github.com/sierrasoftworks/git-tool"
        );
        assert_eq!(
            render("{{ .Repo.GitURL }}", context.clone()).unwrap(),
            "git@github.com:sierrasoftworks/git-tool.git"
        );
        assert_eq!(
            render("{{ .Repo.HttpURL }}", context.clone()).unwrap(),
            "git@github.com:sierrasoftworks/git-tool.git"
        );

        assert_eq!(
            render("{{ .Target.Name }}", context.clone()).unwrap(),
            "sierrasoftworks/git-tool"
        );
        assert_eq!(
            render("{{ .Target.Path }}", context.clone()).unwrap(),
            "/test/github.com/sierrasoftworks/git-tool"
        );

        assert_eq!(
            render("{{ .Service.Domain }}", context.clone()).unwrap(),
            "gh"
        );
        assert_eq!(
            render("{{ .Repo.Service.Domain }}", context.clone()).unwrap(),
            "gh"
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
    fn render_invalid_field() {
        let scratch = Scratchpad::new("2020w07", PathBuf::from("/test/scratch/2020w07"));

        render("{{ .Target.UnknownField }}", (&scratch).into()).unwrap_err();
    }
}
