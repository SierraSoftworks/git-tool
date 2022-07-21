use super::{errors, Error};
use crate::core::Core;

#[tracing::instrument(err, skip(core, content))]
pub async fn add_or_update(
    core: &Core,
    content: &str,
    new_languages: Vec<&str>,
) -> Result<String, Error> {
    let mut managed: GitIgnoreFileSection;

    match GitIgnoreFileSection::parse(content) {
        Some(parsed) => {
            managed = parsed;

            managed
                .languages
                .extend(new_languages.iter().map(|x| x.to_string()));
        }
        None => {
            managed = GitIgnoreFileSection {
                content: "".to_string(),
                languages: new_languages.iter().map(|x| x.to_string()).collect(),
                prologue: content.to_string(),
            };
        }
    }

    managed.languages.dedup();
    managed.content = ignore(core, managed.languages.iter().map(|x| x.as_str()).collect()).await?;

    Ok(managed.into())
}

#[tracing::instrument(err, skip(core))]
pub async fn list(core: &Core) -> Result<Vec<String>, Error> {
    let uri = "https://www.toptal.com/developers/gitignore/api/list"
        .parse()
        .map_err(|e| {
            errors::system_with_internal(
                "Could not parse gitignore.io URL.",
                "Please report this error to us by creating an issue on GitHub.",
                e,
            )
        })?;
    let response = core.http_client().get(uri).await?;

    if !response.status().is_success() {
        return Err(response.into());
    }

    let content = response.text().await?;
    Ok(content
        .split(|c: char| c == ',' || c == '\n')
        .map(|slice| String::from_utf8(Vec::from(slice)).unwrap_or_default())
        .collect())
}

#[tracing::instrument(err, skip(core))]
pub async fn ignore(core: &Core, langs: Vec<&str>) -> Result<String, Error> {
    if langs.is_empty() {
        return Ok("".to_string());
    }

    let uri = format!(
        "https://www.toptal.com/developers/gitignore/api/{}",
        langs.join(",")
    )
    .parse()
    .map_err(|e| {
        errors::system_with_internal(
            "Could not parse gitignore.io URL.",
            "Please report this error to us by creating an issue on GitHub.",
            e,
        )
    })?;
    let response = core.http_client().get(uri).await?;

    if response.status() == reqwest::StatusCode::NOT_FOUND {
        return Err(errors::user(
            "We could not find one of the languages you requested.",
            "Check that the languages you've provided are all available using the 'gt ignore' command."));
    }

    if !response.status().is_success() {
        return Err(response.into());
    }

    Ok(response.text().await?)
}

struct GitIgnoreFileSection {
    prologue: String,
    languages: Vec<String>,
    content: String,
}

#[allow(clippy::from_over_into)]
impl std::convert::Into<String> for GitIgnoreFileSection {
    fn into(self) -> String {
        if self.languages.is_empty() {
            return self.prologue;
        }

        format!("{}\n## -------- Managed by Git Tool -------- ##\n## Add any custom rules above this block ##\n## ------------------------------------- ##\n## @languages: {}\n{}", self.prologue, self.languages.join(","), self.content)
    }
}

impl GitIgnoreFileSection {
    fn parse(input: &str) -> Option<GitIgnoreFileSection> {
        let mut has_section = false;
        let mut in_header = true;

        let mut prologue: Vec<String> = Vec::new();
        let mut content: Vec<String> = Vec::new();
        let mut languages: Vec<String> = Vec::new();

        for line in input.split_terminator('\n') {
            if !has_section && line == "## -------- Managed by Git Tool -------- ##" {
                has_section = true;
            }

            if !has_section {
                prologue.push(line.to_string());
                continue;
            }

            if !in_header || !line.starts_with("##") {
                in_header = false;
                content.push(line.to_string());
                continue;
            }

            if let Some(langs) = line.strip_prefix("## @languages: ") {
                languages = langs.split(',').map(|x| x.trim().to_string()).collect();
            }
        }

        if !has_section {
            None
        } else {
            Some(GitIgnoreFileSection {
                prologue: prologue.join("\n").trim().to_string(),
                content: content.join("\n").trim().to_string(),
                languages,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Core;

    #[tokio::test]
    async fn get_list() {
        let core = Core::builder().build();
        match list(&core).await {
            Ok(items) => {
                assert!(!items.is_empty());

                for item in items {
                    assert_ne!(item, String::from(""));
                }
            }
            Err(err) => panic!("{}", err.message()),
        }
    }

    #[tokio::test]
    async fn get_ignore() {
        let core = Core::builder().build();
        match ignore(&core, vec!["csharp"]).await {
            Ok(ignore) => {
                assert_ne!(ignore, String::from(""));
            }
            Err(err) => panic!("{}", err.message()),
        }
    }

    #[test]
    fn parse_section_start_of_file() {
        match GitIgnoreFileSection::parse(
            "
## -------- Managed by Git Tool -------- ##
## Add any custom rules above this block ##
## ------------------------------------- ##
## @languages: go,rust, csharp
*.exe",
        ) {
            Some(section) => {
                assert_eq!(section.languages, vec!["go", "rust", "csharp"]);
                assert_eq!(section.prologue, "");
                assert_eq!(section.content, "*.exe");
            }
            None => panic!("We should have parsed the section correctly"),
        }
    }

    #[test]
    fn parse_section_end_of_file() {
        match GitIgnoreFileSection::parse(
            "
junit.xml
bin/

## -------- Managed by Git Tool -------- ##
## Add any custom rules above this block ##
## ------------------------------------- ##
## @languages: csharp,java
*.exe
*.obj",
        ) {
            Some(section) => {
                assert_eq!(section.languages, vec!["csharp", "java"]);
                assert_eq!(section.prologue, "junit.xml\nbin/");
                assert_eq!(section.content, "*.exe\n*.obj");
            }
            None => panic!("We should have parsed the section correctly"),
        }
    }

    #[test]
    fn parse_section_missing() {
        if let Some(_section) = GitIgnoreFileSection::parse(
            "
junit.xml
bin/
*.exe
*.obj",
        ) {
            panic!("we should not have parsed a section")
        }
    }

    #[tokio::test]
    async fn add_or_update_empty() {
        let core = Core::builder().build();
        match add_or_update(&core, "", vec!["rust"]).await {
            Ok(result) => {
                assert!(result.contains("## @languages: rust\n"));
                assert!(result.contains("target/\n"));
            }
            Err(e) => panic!("{}", e.message()),
        }
    }

    #[tokio::test]
    async fn add_or_update_no_languages() {
        let core = Core::builder().build();
        match add_or_update(&core, "", vec![]).await {
            Ok(result) => {
                assert_eq!(result, "");
            }
            Err(e) => panic!("{}", e.message()),
        }
    }

    #[tokio::test]
    async fn add_or_update_invalid_language() {
        let core = Core::builder().build();
        match add_or_update(&core, "", vec!["thisisnotareallanguage"]).await {
            Ok(_result) => {
                panic!("It should return an error, not succeed");
            }
            Err(e) => {
                assert_eq!(e.message(), "Oh no! We could not find one of the languages you requested.\n\nTo try and fix this, you can:\n - Check that the languages you've provided are all available using the 'gt ignore' command.");
            }
        }
    }

    #[tokio::test]
    async fn add_or_update_existing_unmanaged() {
        let core = Core::builder().build();
        match add_or_update(&core, "/tmp", vec!["rust"]).await {
            Ok(result) => {
                assert!(result.contains("## @languages: rust\n"));
                assert!(result.contains("target/\n"));
                assert!(result.contains("/tmp\n"));
            }
            Err(e) => panic!("{}", e.message()),
        }
    }

    #[tokio::test]
    async fn add_or_update_existing_same_langs() {
        let core = Core::builder().build();
        match add_or_update(
            &core,
            "
## -------- Managed by Git Tool -------- ##
## Add any custom rules above this block ##
## ------------------------------------- ##
## @languages: go,rust
/test
",
            vec!["rust"],
        )
        .await
        {
            Ok(result) => {
                assert!(result.contains("## @languages: go,rust\n"));
                assert!(result.contains("target/\n"));
                assert!(!result.contains("/test\n"));
            }
            Err(e) => panic!("{}", e.message()),
        }
    }

    #[tokio::test]
    async fn add_or_update_existing_new_langs() {
        let core = Core::builder().build();
        match add_or_update(
            &core,
            "
## -------- Managed by Git Tool -------- ##
## Add any custom rules above this block ##
## ------------------------------------- ##
## @languages: go
/test
",
            vec!["rust"],
        )
        .await
        {
            Ok(result) => {
                assert!(result.contains("## @languages: go,rust\n"));
                assert!(result.contains("target/\n"));
                assert!(!result.contains("/test\n"));
            }
            Err(e) => panic!("{}", e.message()),
        }
    }
}
