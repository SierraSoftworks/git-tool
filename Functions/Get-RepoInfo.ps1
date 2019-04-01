<#
    .SYNOPSIS
        Gets an object describing the full details of a repo based on its name and hosting service.

    .DESCRIPTION
        Generates an object which includes information about a repo including its expected location on
        your local filesystem, the webpage on which it may be viewed, the URL which should be used by
        a git clone operation and more.

    .PARAMETER Repo
        The name of the repository which you would like to describe.

    .PARAMETER Service
        The remote hosting provider for the repository in question.
    
    .PARAMETER Path
        The directory within which all of your repositories are stored. Defaults to the value of $GitTool.Directory
        if not specified.
    
    .EXAMPLE
        Get-RepoInfo sierrasoftworks/git-tool

        Name                           Value
        ----                           -----
        WebURL                         https://github.com/SierraSoftworks/git-tool
        Service                        github.com
        Repo                           SierraSoftworks/git-tool
        Path                           C:\dev\github.com\SierraSoftworks\git-tool
        Exists                         True
        GitURL                         git@github.com:SierraSoftworks/git-tool.git
#>
function Get-RepoInfo {
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

    $repoUrl = "https://$Service/$Repo"
    $gitUrl = "git@${Service}:$Repo.git"
    $repoPath = [System.IO.Path]::Combine($Path, $Service, $Repo.Replace('/', [System.IO.Path]::DirectorySeparatorChar))

    if ($Service -eq "dev.azure.com") {
        $repoNameSplit = $Repo -split '/'
        $repoNamespace = $repoNameSplit[0..($repoNameSplit.Length - 2)] -join '/'
        $repoName = $repoNameSplit[-1]

        $repoUrl = "https://$Service/${repoNamespace}/_git/${repoName}"
        $gitUrl = "git@ssh.dev.azure.com:v3/$Repo"
    }

    return @{
        Repo    = $Repo;
        Service = $Service;
        Path    = $repoPath;
        WebURL  = $repoUrl;
        GitURL  = $gitUrl;
        Exists  = $(Test-Path -Path $repoPath -PathType Container);
    }
}

Register-ArgumentCompleter -CommandName Get-RepoInfo -ParameterName Repo -ScriptBlock $Function:SuggestRepoName