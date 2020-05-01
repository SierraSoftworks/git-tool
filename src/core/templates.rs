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
                "Namespace" => Value::String(self.get_namespace())
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
            })
        })
        
    }
}