<#
    .SYNOPSIS
        Creates a new local repository with the provided name.

    .DESCRIPTION
        Used to quickly bootstrap a local repository by automatically creating the correct
        folder structure, configuring your Git remotes and optionally adding a relevant gitignore
        file. Upon completion, you may switch to the location of the newly created repo by either
        passing the -Open flag or by using Open-Repo.

    .PARAMETER Repo
        The name of the repository which you would like to fetch.

    .PARAMETER Service
        The remote hosting provider from which you would like to fetch the repository.
    
    .PARAMETER Path
        The directory within which all of your repositories are stored. Defaults to the value of $GitTool.Directory
        if not specified.

    .PARAMETER Open
        Whether you would like to reconfigure the local repository to use the updated remote address in situations where that has changed.

    .PARAMETER GitIgnore
        The language or tool for which you would like to fetch a gitignore file. If not provided, defaults to your globally configured
        $GitTool.GitIgnore.Default option.
    
    .EXAMPLE
        New-Repo sierrasoftworks/git-tool

        Running git init
        Running git remote add origin git@github.com:SierraSoftworks/git-tool.git

        Name                           Value
        ----                           -----
        WebURL                         https://github.com/SierraSoftworks/git-tool
        Service                        github.com
        Repo                           SierraSoftworks/git-tool
        Path                           C:\dev\github.com\SierraSoftworks\git-tool
        Exists                         True
        GitURL                         git@github.com:SierraSoftworks/git-tool.git
    
    .EXAMPLE
        New-Repo sierrasoftworks/git-tool -GitIgnore powershell

        Running git init
        Running git remote add origin git@github.com:SierraSoftworks/git-tool.git
        Adding .gitignore file

        Name                           Value
        ----                           -----
        WebURL                         https://github.com/SierraSoftworks/git-tool
        Service                        github.com
        Repo                           SierraSoftworks/git-tool
        Path                           C:\dev\github.com\SierraSoftworks\git-tool
        Exists                         True
        GitURL                         git@github.com:SierraSoftworks/git-tool.git
#>
function New-Repo {
    param(
        [string]
        [Parameter(Mandatory = $true, ValueFromPipeline = $true, HelpMessage = "The name of the repository that you wish to synchronize (e.g. Namespace/RepoName)")]
        $Repo,

        [string]
        [Parameter(HelpMessage = "The service hosting your repository (e.g. github.com)")]
        [ValidateSet("github.com", "dev.azure.com", "gitlab.com", "bitbucket.org")]
        $Service = $GitTool.Service,

        [string]
        [Parameter(HelpMessage = "The directory within which your repositories will be checked out (e.g. /src/).")]
        $Path = $GitTool.Directory,

        [switch]
        [Parameter(HelpMessage = "Open the repo once it has been created.")]
        $Open = $false,

        [string]
        [Parameter(HelpMessage = "The language or tool for which you would like to fetch a gitignore file")]
        $GitIgnore = $GitTool.GitIgnore.Default
    )

    $info = Get-RepoInfo -Repo $Repo -Service $Service -Path $Path

    Write-Host -NoNewline "Creating "
    Write-Host -ForegroundColor Blue $info.WebURL
    Write-Host -NoNewline " - Git URL:   "
    Write-Host -ForegroundColor Red $info.GitURL
    Write-Host -NoNewline " - Target:    "
    Write-Host -ForegroundColor Green $info.Path

    if (-not $info.Exists) {
        New-Item -Path $info.Path -ItemType Container | Out-Null
    }

    Write-Host ""

    Push-Location -Path $info.Path
    try {
        if (Test-Path -PathType Container -Path ".git") {
            Write-Error -Category ResourceExists -Message "The repository $Service/$Repo already exists." -RecommendedAction "Please use git to manage this repository directly."
            return
        }
        
        Write-Host -NoNewline "Running "
        Write-Host -ForegroundColor DarkGray "git init"
        git.exe init | Out-Null

        Write-Host -NoNewline "Running "
        Write-Host -ForegroundColor DarkGray "git remote add origin ${info.GitURL}"
        git.exe remote add origin $info.GitURL | Out-Null

        if ($null -ne $GitIgnore) {
            Write-Host "Adding .gitignore file"
            Get-GitIgnore -Language $GitIgnore | Set-Content .gitignore
        }
    }
    finally {
        Pop-Location
    }

    if ($Open) {
        Open-Repo -Repo $Repo -Service $Service -Path $Path
    }

    return $info
}

Register-ArgumentCompleter -CommandName New-Repo -ParameterName Repo -ScriptBlock $Function:SuggestRepoPrefix
Register-ArgumentCompleter -CommandName New-Repo -ParameterName GitIgnore -ScriptBlock $Function:SuggestGitIgnoreType