#requires -version 5.1

. $PSScriptRoot\Config.ps1

Get-ChildItem -Path $PSScriptRoot\Functions\*.ps1 | ForEach-Object {
    . $_.FullName
}

. $PSScriptRoot\Autocomplete.ps1

@("Get-GitIgnore", "New-Repo") | ForEach-Object {
    Register-ArgumentCompleter -CommandName $_ -ParameterName GitIgnore -ScriptBlock $Function:SuggestAutocomplete
}

@("Get-Repo", "Get-RepoInfo", "New-Repo", "Open-Repo") | ForEach-Object {
    Register-ArgumentCompleter -CommandName $_ -ParameterName Repo -ScriptBlock $Function:SuggestAutocomplete
}