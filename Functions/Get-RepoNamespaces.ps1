<#
    .SYNOPSIS
        Gets a list of all the known repository namespaces available for a certain hosting service.

    .DESCRIPTION
        Lists all of the namespaces for a given service for which you have local repositories. These
        namespaces form the first component of a repository name (i.e. namespace/name) and this information
        will primarily be used to provide autocompletion hints for the New-Repo command.

    .PARAMETER Service
        The remote hosting provider for the repository namespaces in question.
    
    .PARAMETER Path
        The directory within which all of your repositories are stored. Defaults to the value of $GitTool.Directory
        if not specified.
    
    .EXAMPLE
        Get-RepoNamespaces
        SierraSoftworks
        spartan563

    .LINK
        New-Repo
#>
function Get-RepoNamespaces {
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
    for ($i = 1; $i -lt $GitTool.Services[$Service].NamespaceDepth; $i++) {
        $matcher = "$matcher/*"
    }

    Get-ChildItem -Path $servicePath -Directory -Depth ($GitTool.Services[$Service].NamespaceDepth - 1) | ForEach-Object {
        $_.FullName.Substring($servicePath.Length).Replace([System.IO.Path]::DirectorySeparatorChar, "/").Trim("/")
    } | Sort-Object -Unique | Where-Object { $_ -like $matcher }
}