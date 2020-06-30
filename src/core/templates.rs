use gtmpl::{template, Value};
use super::{Repo, Target, errors, Scratchpad};

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
        e))
}

impl<'a> std::convert::Into<Value> for &Repo {
    fn into(self) -> Value {
        Value::Object(map!{
            "Target" => Value::Object(map!{
                "Name" => Value::String(self.get_full_name()),
                "Path" => Value::String(String::from(self.get_path().to_str().unwrap_or_default())),
                "Exists" => Value::Bool(self.exists())
            }),
            "Repo" => Value::Object(map!{
                "FullName" => Value::String(self.get_full_name()),
                "Name" => Value::String(self.get_name()),
                "Namespace" => Value::String(self.get_namespace()),
                "Domain" => Value::String(self.get_domain()),
                "Exists" => Value::Bool(self.exists()),
                "Path" => Value::String(String::from(self.get_path().to_str().unwrap_or_default())),
                "Service" => Value::Object(map!{
                    "Domain" => Value::String(self.get_domain())
                })
            }),
            "Service" => Value::Object(map!{
                "Domain" => Value::String(self.get_domain())
            })
        })
        
    }
}

impl std::convert::Into<Value> for &Scratchpad {
    fn into(self) -> Value {
        Value::Object(map!{
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
        let repo = Repo::new(
            "github.com/sierrasoftworks/git-tool", 
            PathBuf::from("/test/github.com/sierrasoftworks/git-tool"));

        assert_eq!(render("{{ .Repo.Name }}", (&repo).into()).unwrap(), "git-tool");
        assert_eq!(render("{{ .Repo.FullName }}", (&repo).into()).unwrap(), "sierrasoftworks/git-tool");
        assert_eq!(render("{{ .Repo.Namespace }}", (&repo).into()).unwrap(), "sierrasoftworks");
        assert_eq!(render("{{ .Repo.Domain }}", (&repo).into()).unwrap(), "github.com");
        assert_eq!(render("{{ .Repo.Path }}", (&repo).into()).unwrap(), "/test/github.com/sierrasoftworks/git-tool");

        assert_eq!(render("{{ .Target.Name }}", (&repo).into()).unwrap(), "sierrasoftworks/git-tool");
        assert_eq!(render("{{ .Target.Path }}", (&repo).into()).unwrap(), "/test/github.com/sierrasoftworks/git-tool");

        assert_eq!(render("{{ .Service.Domain }}", (&repo).into()).unwrap(), "github.com");
    }

    #[test]
    fn render_basic_scratchpad() {
        let scratch = Scratchpad::new(
            "2020w07", 
            PathBuf::from("/test/scratch/2020w07"));

        assert_eq!(render("{{ .Target.Name }}", (&scratch).into()).unwrap(), "2020w07");
        assert_eq!(render("{{ .Target.Path }}", (&scratch).into()).unwrap(), "/test/scratch/2020w07");
    }

    #[test]
    fn render_advanced_repo() {
        let repo = Repo::new(
            "github.com/sierrasoftworks/git-tool", 
            PathBuf::from("/test/github.com/sierrasoftworks/git-tool"));

        assert_eq!(render("{{ with .Repo }}{{ .Service.Domain }}/{{ .FullName }}{{ else }}{{ .Target.Name }}{{ end }}", (&repo).into()).unwrap(), "github.com/sierrasoftworks/git-tool");
    }

    #[test]
    fn render_advanced_scratchpad() {
        let scratch = Scratchpad::new(
            "2020w07", 
            PathBuf::from("/test/scratch/2020w07"));

        assert_eq!(render("{{ with .Repo }}{{ .Service.Domain }}/{{ .FullName }}{{ else }}{{ .Target.Name }}{{ end }}", (&scratch).into()).unwrap(), "2020w07");
    }

    #[test]
    fn render_invalid_syntax() {
        let scratch = Scratchpad::new(
            "2020w07", 
            PathBuf::from("/test/scratch/2020w07"));

        render("{{ .Target.Name", (&scratch).into()).unwrap_err();
    }

    #[test]
    fn render_invalid_field() {
        let scratch = Scratchpad::new(
            "2020w07", 
            PathBuf::from("/test/scratch/2020w07"));

        render("{{ .Target.UnknownField }}", (&scratch).into()).unwrap_err();
    }
}