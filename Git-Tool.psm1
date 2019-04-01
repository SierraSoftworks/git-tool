#requires -version 5.1

. $PSScriptRoot\Config.ps1

Get-ChildItem -Path $PSScriptRoot\Functions\*.ps1 | ForEach-Object {
    . $_.FullName
}
