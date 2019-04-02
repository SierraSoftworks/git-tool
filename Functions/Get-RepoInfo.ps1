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
        $Service = $GitTool.Service,

        [string]
        [Parameter(HelpMessage = "The directory within which your repositories will be checked out (e.g. /src/)")]
        $Path = $GitTool.Directory
    )

    $repoPath = [System.IO.Path]::Combine($Path, $Service, $Repo.Replace('/', [System.IO.Path]::DirectorySeparatorChar))
    
    $repoNameSplit = $Repo -split '/'

    if ($repoNameSplit.Length -ne ($GitTool.Services[$Service].NamespaceDepth + 1)) {
        Write-Error -Category InvalidArgument -Message "The service $Service expects a repository name with $($GitTool.Services[$Service].NamespaceDepth + 1) path segments, you provided $($repoNameSplit.Length) segments." -RecommendedAction "Please enter a name with $($GitTool.Services[$Service].NamespaceDepth + 1) path segments each separated by a '/'."
        return
    }

    $repoNamespace = $repoNameSplit[0..($GitTool.Services[$Service].NamespaceDepth - 1)] -join '/'
    $repoName = $repoNameSplit[$GitTool.Services[$Service].NamespaceDepth]

    $repoUrl = [System.String]::Format($GitTool.Services[$Service].WebURLFormat, $repoNamespace, $repoName)
    $gitUrl = [System.String]::Format($GitTool.Services[$Service].GitURLFormat, $repoNamespace, $repoName)

    return @{
        Repo      = "$repoNamespace/$repoName";
        Namespace = $repoNamespace;
        Name      = $repoName;
        Service   = $Service;
        Path      = $repoPath;
        WebURL    = $repoUrl;
        GitURL    = $gitUrl;
        Exists    = $(Test-Path -Path $repoPath -PathType Container);
    }
}
