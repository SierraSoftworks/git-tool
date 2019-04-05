<#
    .SYNOPSIS
        Fetches a gitignore file from gitignore.io for the given language.

    .DESCRIPTION
        Uses the gitignore.io API to download the contents of a gitignore file
        which provides sensible defaults for the given language. Can be used to quickly
        bootstrap a repository or add a gitignore file to an existing one.

    .PARAMETER Language
        The programming language or tool name for which a gitignore file will be retrieved.

    .LINK
        https://gitignore.io

    .EXAMPLE
        Get-GitIgnore powershell | Set-Content .gitignore
#>
function Get-GitIgnore {
    param (
        [Parameter(HelpMessage = "The programming language to get a gitignore file for")]
        [string]
        $Language = $GitTool.GitIgnore.Default
    )

    (Invoke-WebRequest "https://gitignore.io/api/$Language" -UseBasicParsing).Content
}

function Get-GitIgnoreTypes {
    (Invoke-WebRequest "https://gitignore.io/api/list" -UseBasicParsing).Content.Split(',', "`t", "`n", "`r") | ForEach-Object { $_.Trim() }
}