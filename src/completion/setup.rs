use std::env::args;

pub struct Shell {
    name: &'static str,
    short_init: String,
    long_init: String,
}

impl Shell {
    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_short_init(&self) -> &str {
        &self.short_init
    }

    pub fn get_long_init(&self) -> &str {
        &self.long_init
    }
}

pub fn get_shells() -> Vec<Shell> {
    vec![
        Shell {
            name: "powershell",
            short_init: format!(
                r#"Invoke-Expression (@(&"{app}" shell-init powershell --full) -join "`n")"#,
                app = args().next().unwrap_or("git-tool".to_string())
            ),
            long_init: format!(
                r#"
Register-ArgumentCompleter -CommandName gt, git-tool, git-tool.exe -ScriptBlock {{
param([string]$commandName, [string]$wordToComplete, [int]$cursorPosition)

&"{app}" complete --position $cursorPosition "$wordToComplete" | ForEach-Object {{
    [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_)
}}
}} -Native
            "#,
                app = args().next().unwrap_or("git-tool".to_string())
            ),
        },
        Shell {
            name: "bash",
            short_init: format!(
                r#"
if [ "${{BASH_VERSINFO[0]}}" -gt 4 ] || ([ "${{BASH_VERSINFO[0]}}" -eq 4 ] && [ "${{BASH_VERSINFO[1]}}" -ge 1 ])
then
source <("{app}" shell-init bash --full)
else
source /dev/stdin <<<"$("%s" shell-init bash --full)"
fi
            "#,
                app = args().next().unwrap_or("git-tool".to_string())
            ),
            long_init: format!(
                r#"
_gittool_bash_autocomplete() {{
    local word=${{COMP_WORDS[COMP_CWORD]}}

    local completions
    completions="$({app} complete --position "${{COMP_POINT}}" "${{COMP_LINE}}" 2>/dev/null)"
    if [ $? -ne 0 ]; then
        completions=""
    fi

    COMPREPLY=( $(compgen -W "$completions" -- "$word") )
}}

complete -F _gittool_bash_autocomplete gt git-tool
            "#,
                app = args().next().unwrap_or("git-tool".to_string())
            ),
        },
        Shell {
            name: "zsh",
            short_init: format!(
                r#"source <("{app}" shell-init zsh --full)"#,
                app = args().next().unwrap_or("git-tool".to_string())
            ),
            long_init: format!(
                r#"
_gittool_zsh_autocomplete() {{
    local completions=("$({app} complete "$words")")

    reply=( "${{(ps:\n:)completions}}" )
}}
    
compctl -U -K _gittool_zsh_autocomplete git-tool
            "#,
                app = args().next().unwrap_or("git-tool".to_string())
            ),
        },
        Shell {
            name: "fish",
            short_init: format!(
                r#"complete -f -c {app} "({app} complete)"#,
                app = args().next().unwrap_or("git-tool".to_string())
            ),
            long_init: format!(
                r#"complete -f -c {app} "({app} complete)"#,
                app = args().next().unwrap_or("git-tool".to_string())
            ),
        },
    ]
}
