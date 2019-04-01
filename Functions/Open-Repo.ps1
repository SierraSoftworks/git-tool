<#
    .SYNOPSIS
        Changes your location to the directory of the repository you have specified.

    .DESCRIPTION
        Quickly changes the location of your session to the directory holding the
        repository you have specified, if it exists.

    .PARAMETER Repo
        The name of the repository which you would like to open.

    .PARAMETER Service
        The remote hosting provider from which you would like to open the repository.

    .PARAMETER Path
        The directory within which all of your repositories are stored. Defaults to the value of $GitTool.Directory
        if not specified.

    .EXAMPLE
        Open-Repo sierrasoftworks/git-tool

        Name                           Value
        ----                           -----
        WebURL                         https://github.com/SierraSoftworks/git-tool
        Service                        github.com
        Repo                           SierraSoftworks/git-tool
        Path                           C:\dev\github.com\SierraSoftworks\git-tool
        Exists                         True
        GitURL                         git@github.com:SierraSoftworks/git-tool.git

        C:\dev\github.com\SierraSoftworks\git-tool PS> 
#>
function Open-Repo {
    param(
        [string]
        [Parameter(Mandatory = $true, ValueFromPipeline = $true, HelpMessage = "The name of the repository that you wish to synchronize (e.g. Namespace/RepoName)")]
        $Repo,

        [string]
        [Parameter(HelpMessage = "The service hosting your repository (e.g. github.com)")]
        [ValidateSet("github.com", "dev.azure.com", "gitlab.com", "bitbucket.org")]
        $Service = $GitTool.Service,

        [string]
        [Parameter(HelpMessage = "The directory within which your repositories will be checked out (e.g. /src/)")]
        $Path = $GitTool.Directory
    )

    $info = Get-RepoInfo -Repo $Repo -Service $Service -Path $Path

    if (-not $info.Exists) {
        Write-Error -Category ResourceExists -Message "The repository $Service/$Repo does not exist." -RecommendedAction "Use New-Repo to create it."
        return
    }

    Set-Location -Path $info.Path

    return $info
}

Register-ArgumentCompleter -CommandName Open-Repo -ParameterName Repo -ScriptBlock $Function:SuggestRepoName