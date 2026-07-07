use std::env::args;

use crate::engine::Core;

pub trait Shell {
    fn name(&self) -> &'static str;
    fn short_init(&self) -> String;
    fn long_init(&self, core: &Core) -> String;

    fn config_file(&self) -> &'static str;
    fn install(&self) -> String;
}

pub fn get_shells() -> Vec<Box<dyn Shell>> {
    vec![
        Box::new(PowerShell),
        Box::new(Bash),
        Box::new(Zsh),
        Box::new(Fish),
    ]
}

fn app_path() -> String {
    args().next().unwrap_or_else(|| "git-tool".to_string())
}

struct PowerShell;
impl Shell for PowerShell {
    fn name(&self) -> &'static str {
        "powershell"
    }

    fn short_init(&self) -> String {
        let app = app_path();
        format!(r#"Invoke-Expression (@(&"{app}" shell-init powershell --full) -join "`n")"#)
    }

    fn long_init(&self, core: &Core) -> String {
        let app = app_path();
        let session_id = core.analytics().session_id();

        format!(
            r#"
Register-ArgumentCompleter -CommandName gt, git-tool, git-tool.exe -ScriptBlock {{
param([string]$commandName, [string]$wordToComplete, [int]$cursorPosition)

&"{app}" complete --position $cursorPosition "$wordToComplete" | ForEach-Object {{
    [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_)
}}
}} -Native

$env:GITTOOL_SESSION_ID = "{session_id}"
"#
        )
    }

    fn config_file(&self) -> &'static str {
        "$PROFILE.CurrentUserAllHosts"
    }

    fn install(&self) -> String {
        let app = app_path();
        format!(
            r#"Invoke-Expression (&{app} shell-init powershell)
New-Alias -Name gt -Value {app}"#
        )
    }
}

struct Bash;
impl Shell for Bash {
    fn name(&self) -> &'static str {
        "bash"
    }

    fn short_init(&self) -> String {
        let app = app_path();
        format!(
            r#"
if [ "${{BASH_VERSINFO[0]}}" -gt 4 ] || ([ "${{BASH_VERSINFO[0]}}" -eq 4 ] && [ "${{BASH_VERSINFO[1]}}" -ge 1 ])
then
source <("{app}" shell-init bash --full)
else
source /dev/stdin <<<"$("%s" shell-init bash --full)"
fi
"#
        )
    }

    fn long_init(&self, core: &Core) -> String {
        let app = app_path();
        let session_id = core.analytics().session_id();

        format!(
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

export GITTOOL_SESSION_ID="{session_id}"
"#
        )
    }

    fn config_file(&self) -> &'static str {
        "~/.bashrc"
    }

    fn install(&self) -> String {
        let app = app_path();
        format!(
            r#"eval "$({app} shell-init bash)"
alias gt={app}"#
        )
    }
}

struct Zsh;
impl Shell for Zsh {
    fn name(&self) -> &'static str {
        "zsh"
    }

    fn short_init(&self) -> String {
        let app = app_path();
        format!(r#"source <("{app}" shell-init zsh --full)"#)
    }

    fn long_init(&self, core: &Core) -> String {
        let app = app_path();
        let session_id = core.analytics().session_id();

        format!(
            r#"
_gittool_zsh_autocomplete() {{
    local completions=("$({app} complete "$words")")

    reply=( "${{(ps:\n:)completions}}" )
}}
    
compctl -U -K _gittool_zsh_autocomplete git-tool

export GITTOOL_SESSION_ID="{session_id}"
"#
        )
    }

    fn config_file(&self) -> &'static str {
        "~/.zshrc"
    }

    fn install(&self) -> String {
        let app = app_path();
        format!(
            r#"eval "$({app} shell-init zsh)"
alias gt={app}"#
        )
    }
}

struct Fish;
impl Shell for Fish {
    fn name(&self) -> &'static str {
        "fish"
    }

    fn short_init(&self) -> String {
        let app = app_path();
        format!(r#"source ("{app}" shell-init fish --full | psub)"#)
    }

    fn long_init(&self, core: &Core) -> String {
        let app = app_path();
        let session_id = core.analytics().session_id();

        format!(
            r#"
complete -f -c {app} "({app} complete (commandline -cp))"
set -gx GITTOOL_SESSION_ID "{session_id}"
"#
        )
    }

    fn config_file(&self) -> &'static str {
        "~/.fishrc"
    }

    fn install(&self) -> String {
        let app = app_path();
        format!(
            r#"eval "$({app} shell-init fish)"
alias gt={app}"#
        )
    }
}
