<#
    .SYNOPSIS
        Gets the latest version of a particular repository from the upstream hosting provider.

    .DESCRIPTION
        Used to quickly acquire the latest version of a given repository from a remote hosting provider.
        Will automatically execute a git clone operation if necessary, falling back to git pull if the
        repository already exists locally.

    .PARAMETER Repo
        The name of the repository which you would like to fetch.

    .PARAMETER Service
        The remote hosting provider from which you would like to fetch the repository.

    .PARAMETER Branch
        The repository branch which you owuld like to checkout.
    
    .PARAMETER Path
        The directory within which all of your repositories are stored. Defaults to the value of $GitTool.Directory
        if not specified.

    .PARAMETER Reconfigure
        Whether you would like to reconfigure the local repository to use the updated remote address in situations where that has changed.
    
    .PARAMETER Open
        Whether or not to open the repository once it has been fetched.

    .EXAMPLE
        Get-Repo sierrasoftworks/git-tool

        Synchronizing https://github.com/SierraSoftworks/git-tool
        - Git URL:   git@github.com:SierraSoftworks/git-tool.git
        - Target:    C:\dev\github.com\SierraSoftworks\git-tool
        Running git pull
        Already up to date.

        Name                           Value
        ----                           -----
        WebURL                         https://github.com/SierraSoftworks/git-tool
        Service                        github.com
        Repo                           SierraSoftworks/git-tool
        Path                           C:\dev\github.com\SierraSoftworks\git-tool
        Exists                         True
        GitURL                         git@github.com:SierraSoftworks/git-tool.git
#>
function Get-Repo {
    param(
        [string]
        [Parameter(Mandatory = $true, ValueFromPipeline = $true, HelpMessage = "The name of the repository that you wish to synchronize (e.g. Namespace/RepoName)")]
        $Repo,

        [string]
        [Parameter(HelpMessage = "The service hosting your repository (e.g. github.com)")]
        [ValidateSet("github.com", "dev.azure.com", "gitlab.com", "bitbucket.org")]
        $Service = $GitTool.Service,

        [string]
        [Parameter(HelpMessage = "The branch to checkout and update (optional).")]
        $Branch = $null,

        [string]
        [Parameter(HelpMessage = "The directory within which your repositories will be checked out (e.g. /src/)")]
        $Path = $GitTool.Directory,

        [switch]
        [Parameter(HelpMessage = "Reconfigure existing local repositories to ensure they use the correct remotes.")]
        $Reconfigure = $false,

        [switch]
        [Parameter(HelpMessage = "Whether to open the repository once it has been updated or not.")]
        $Open = $false
    )

    $info = Get-RepoInfo -Repo $Repo -Service $Service -Path $Path

    Write-Host ""

    Write-Host -NoNewline "Synchronizing "
    Write-Host -ForegroundColor Blue $info.WebURL
    Write-Host -NoNewline " - Git URL:   "
    Write-Host -ForegroundColor Red $info.GitURL
    Write-Host -NoNewline " - Target:    "
    Write-Host -ForegroundColor Green $info.Path

    if (-not $info.Exists) {
        Write-Host -NoNewline "Running "
        Write-Host -ForegroundColor DarkGray "git clone ${info.GitURL} ${info.Path}"
        git.exe clone $info.GitURL $info.Path
    }
    else {
        Push-Location -Path $info.Path
        try {
            if ($Reconfigure) {
                Write-Host -NoNewline "Running "
                Write-Host -ForegroundColor DarkGray "git remote set-url origin ${info.GitURL}"
                git.exe remote set-url origin $info.GitURL
            }
            
            Write-Host -NoNewline "Running "
            Write-Host -ForegroundColor DarkGray "git pull"
            git.exe pull
        }
        finally {
            Pop-Location
        }
    }

    Push-Location -Path $info.Path
    try {
        if ("" -ne "$Branch") {
            Write-Host -NoNewline "Running "
            Write-Host -ForegroundColor DarkGray "git checkout $Branch"
            git.exe checkout $Branch
        }
    }
    finally {
        Pop-Location
    }

    if ($Open) {
        Set-Location -Path $info.Path
    }

    return $info
}