package autocomplete

import (
	"fmt"
	"os"
	"strings"
)

type shell struct {
	Name      string
	ShortInit func() string
	FullInit  func() string
}

func stringIdentity(input string) func() string {
	return func() string {
		return input
	}
}

var shells = []shell{
	shell{
		Name: "powershell",
		ShortInit: func() string {
			return fmt.Sprintf("Invoke-Expression (@(&\"%s\" shell-init powershell --full) -join \"`n\")", os.Args[0])
		},
		FullInit: stringIdentity(`Register-ArgumentCompleter -CommandName gt, git-tool, git-tool.exe -ScriptBlock {
			param([string]$commandName, [string]$wordToComplete, [int]$cursorPosition)
		  
			git-tool.exe complete --position $cursorPosition "$wordToComplete" | ForEach-Object {
			  [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_)
			}
		  } -Native`),
	},
	shell{
		Name: "bash",
		ShortInit: func() string {
			return fmt.Sprintf(`if [ "${BASH_VERSINFO[0]}" -gt 4 ] || ([ "${BASH_VERSINFO[0]}" -eq 4 ] && [ "${BASH_VERSINFO[1]}" -ge 1 ])
			then
			source <("%s" shell-init bash --full)
			else
			source /dev/stdin <<<"$("%s" shell-init bash --full)"
			fi`, os.Args[0], os.Args[0])
		},
		FullInit: stringIdentity(`_gittool_bash_autocomplete() {
			local word=${COMP_WORDS[COMP_CWORD]}
		  
			local completions
			completions="$(git-tool complete --position "${COMP_POINT}" "${COMP_LINE}" 2>/dev/null)"
			if [ $? -ne 0 ]; then
			  completions=""
			fi
		  
			COMPREPLY=( $(compgen -W "$completions" -- "$word") )
		  }
		  
		  complete -F _gittool_bash_autocomplete gt git-tool`),
	},
	shell{
		Name: "zsh",
		ShortInit: func() string {
			return fmt.Sprintf(`source <("%s" shell-init zsh --full)`, os.Args[0])
		},
		FullInit: stringIdentity(`_gittool_zsh_autocomplete() {
			local completions=("$(git-tool complete "$words")")
		  
			reply=( "${(ps:\n:)completions}" )
		  }
			  
		  compdef _gittool_zsh_autocomplete gt git-tool`),
	},
}

// GetInitScript fetches the shell initialization script for the provided shell.
func GetInitScript(shell string) string {
	shell = strings.ToLower(shell)

	for _, s := range shells {
		if s.Name == shell {
			return s.ShortInit()
		}
	}

	return ""
}

// GetFullInitScript fetches the full initialization script for the provided shell.
func GetFullInitScript(shell string) string {
	shell = strings.ToLower(shell)

	for _, s := range shells {
		if s.Name == shell {
			return s.FullInit()
		}
	}

	return ""
}

// GetInitScriptShells fetches the list of supported shells that may be initialized.
func GetInitScriptShells() []string {
	ss := []string{}
	for _, s := range shells {
		ss = append(ss, s.Name)
	}

	return ss
}
