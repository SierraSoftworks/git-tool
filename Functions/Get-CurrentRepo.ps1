<#
    .SYNOPSIS
        Gets the information describing the repository at your current location.

    .DESCRIPTION
        Considers your current location/directory and determines whether it is a valid repository.
        If it is, this will retrieve the repository information describing this repo.

    .PARAMETER Path
        The directory within which all of your repositories are stored. Defaults to the value of $GitTool.Directory
        if not specified.

    .LINK
        Get-RepoInfo

    .EXAMPLE
        Get-CurrentRepo

        Name                           Value
        ----                           -----
        WebURL                         https://github.com/SierraSoftworks/git-tool
        Service                        github.com
        Repo                           SierraSoftworks/git-tool
        Path                           C:\dev\github.com\SierraSoftworks\git-tool
        Exists                         True
        GitURL                         git@github.com:SierraSoftworks/git-tool.git
#>

function Get-CurrentRepo {
    param(
        [string]
        [Parameter(HelpMessage = "The directory within which your repositories will be checked out (e.g. /src/)")]
        $Path = $GitTool.Directory
    )

    $cwd = Get-Location

    if (-not $cwd.Path.StartsWith($Path)) {
        Write-Error -Category ObjectNotFound -Message "You are not in a valid, managed, repository." -RecommendedAction "Switch directories to a managed repository under your $Path directory."
        return
    }

    if (-not (Test-Path -PathType Container -Path ".git")) {
        Write-Error -Category ObjectNotFound -Message "You are not at the root of a valid, managed, repository." -RecommendedAction "Switch directories to the root directory of a git repository."
        return
    }

    $relativePath = $cwd.Path.Substring($Path.Length).Replace([System.IO.Path]::DirectorySeparatorChar, "/").Trim("/")
    $components = $relativePath -split "/"

    $Service = $components[0]
    $Repo = $components[1..$($components.Length)] -join "/"
    
    return Get-RepoInfo -Path $Path -Repo $Repo -Service $Service
}
