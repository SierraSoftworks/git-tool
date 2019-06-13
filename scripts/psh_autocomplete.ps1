New-Alias -Name gt -Value "git-tool.exe"

Register-ArgumentCompleter -CommandName gt, git-tool, git-tool.exe -ScriptBlock {
    param([string]$commandName, [string]$wordToComplete, [int]$cursorPosition)

    git-tool.exe complete --position $cursorPosition "$wordToComplete" | ForEach-Object {
        [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_)
    }
} -Native