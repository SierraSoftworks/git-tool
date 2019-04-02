<#
    .SYNOPSIS
        Gets a list of all the known repositories available for a certain hosting service.

    .DESCRIPTION
        Lists all of the repositories for a given service for which you have local copies. These repositories
        may then be used to perform bulk operations, searches and more. In addition to this, the data reported
        by this command will be used for autocompletion of repository names in commands like Get-Repo, Get-RepoInfo,
        Open-Repo and more.

    .PARAMETER Service
        The remote hosting provider for the repositories in question.
    
    .PARAMETER Path
        The directory within which all of your repositories are stored. Defaults to the value of $GitTool.Directory
        if not specified.
    
    .EXAMPLE
        Get-Repos
        SierraSoftworks/bender
        SierraSoftworks/git-tool
        SierraSoftworks/iridium
        SierraSoftworks/sentry-go
        SierraSoftworks/vue-template

    .LINK
        Get-Repo

    .LINK
        Get-RepoInfo

    .LINK
        Open-Repo
#>
function Get-Repos {
    param (
        [string]
        [Parameter(HelpMessage = "The service hosting your repository (e.g. github.com)")]
        $Service = $GitTool.Service,

        [string]
        [Parameter(HelpMessage = "The directory within which your repositories will be checked out (e.g. /src/)")]
        $Path = $GitTool.Directory
    )

    $servicePath = [System.IO.Path]::Combine($Path, $Service)

    $matcher = "*"
    for ($i = 0; $i -lt $GitTool.Services[$Service].NamespaceDepth; $i++) {
        $matcher = "$matcher/*"
    }

    Get-ChildItem -Path $servicePath -Directory -Depth $GitTool.Services[$Service].NamespaceDepth | ForEach-Object {
        $_.FullName.Substring($servicePath.Length).Replace([System.IO.Path]::DirectorySeparatorChar, "/").Trim("/")
    } | Sort-Object -Unique | Where-Object { $_ -like $matcher }
}