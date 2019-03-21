#requires -version 5.1

function Get-RepoInfo {
    param(
        [string]
        [Parameter(Mandatory = $true, ValueFromPipeline = $true, HelpMessage = "The name of the repository that you wish to synchronize (e.g. Namespace/RepoName)")]
        $Repo,

        [string]
        [Parameter(HelpMessage = "The service hosting your repository (e.g. github.com)")]
        [ValidateSet("github.com", "dev.azure.com", "gitlab.com", "bitbucket.org")]
        $Service = "github.com",

        [string]
        [Parameter(HelpMessage = "The directory within which your repositories will be checked out (e.g. /src/)")]
        $Path = $env:DEV_DIRECTORY
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

function Get-CurrentRepo {
    param(
        [string]
        [Parameter(HelpMessage = "The directory within which your repositories will be checked out (e.g. /src/)")]
        $RootPath = $env:DEV_DIRECTORY
    )

    $cwd = Get-Location

    if (-not $cwd.Path.StartsWith($RootPath)) {
        Write-Error -Category ObjectNotFound -Message "You are not in a valid, managed, repository." -RecommendedAction "Switch directories to a managed repository under your $RootPath directory."
        return
    }

    if (-not (Test-Path -PathType Container -Path ".git")) {
        Write-Error -Category ObjectNotFound -Message "You are not at the root of a valid, managed, repository." -RecommendedAction "Switch directories to the root directory of a git repository."
        return
    }

    $relativePath = $cwd.Path.Substring($RootPath.Length).Replace([System.IO.Path]::DirectorySeparatorChar, "/").Trim("/")
    $components = $relativePath -split "/"

    $Service = $components[0]
    $Repo = $components[1..$($components.Length)] -join "/"
    
    return Get-RepoInfo -Path $RootPath -Repo $Repo -Service $Service
}

function Get-Repo {
    param(
        [string]
        [Parameter(Mandatory = $true, ValueFromPipeline = $true, HelpMessage = "The name of the repository that you wish to synchronize (e.g. Namespace/RepoName)")]
        $Repo,

        [string]
        [Parameter(HelpMessage = "The service hosting your repository (e.g. github.com)")]
        [ValidateSet("github.com", "dev.azure.com", "gitlab.com", "bitbucket.org")]
        $Service = "github.com",

        [string]
        [Parameter(HelpMessage = "The branch to checkout and update (optional).")]
        $Branch = $null,

        [string]
        [Parameter(HelpMessage = "The directory within which your repositories will be checked out (e.g. /src/)")]
        $Path = $env:DEV_DIRECTORY,

        [switch]
        [Parameter(HelpMessage = "Reconfigure existing local repositories to ensure they use the correct remotes.")]
        $Reconfigure = $false
    )

    $info = Get-RepoInfo -Repo $Repo -Service $Service -Path $Path

    Write-Host -NoNewline "Synchronizing "
    Write-Host -ForegroundColor Blue $info.WebURL
    Write-Host -NoNewline " - Git URL:   "
    Write-Host -ForegroundColor Red $info.GitURL
    Write-Host -NoNewline " - Target:    "
    Write-Host -ForegroundColor Green $info.Path

    if ($info.Exists) {
        Push-Location -Path $info.Path
        try {
            if ($Reconfigure) {
                Write-Host -NoNewline "Running "
                Write-Host -ForegroundColor DarkGray "git remote set-url origin ${info.GitURL}"
                git remote set-url origin $info.GitURL
            }
            
            Write-Host -NoNewline "Running "
            Write-Host -ForegroundColor DarkGray "git pull"
            git pull

            if ("" -ne "$Branch") {
                Write-Host -NoNewline "Running "
                Write-Host -ForegroundColor DarkGray "git checkout $Branch"
                git checkout $Branch
            }
        }
        finally {
            Pop-Location
        }
    }
    else {
        Write-Host -NoNewline "Running "
        Write-Host -ForegroundColor DarkGray "git clone ${info.GitURL} ${info.Path}"
        git clone $info.GitURL $info.Path
    }

    return $info
}

function New-Repo {
    param(
        [string]
        [Parameter(Mandatory = $true, ValueFromPipeline = $true, HelpMessage = "The name of the repository that you wish to synchronize (e.g. Namespace/RepoName)")]
        $Repo,

        [string]
        [Parameter(HelpMessage = "The service hosting your repository (e.g. github.com)")]
        [ValidateSet("github.com", "dev.azure.com", "gitlab.com", "bitbucket.org")]
        $Service = "github.com",

        [string]
        [Parameter(HelpMessage = "The directory within which your repositories will be checked out (e.g. /src/).")]
        $Path = $env:DEV_DIRECTORY,

        [switch]
        [Parameter(HelpMessage = "Open the repo once it has been created.")]
        $Open = $false
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

    Push-Location -Path $info.Path
    try {
        if (Test-Path -PathType Container -Path ".git") {
            Write-Error -Category ResourceExists -Message "The repository $Service/$Repo already exists." -RecommendedAction "Please use git to manage this repository directly."
            return
        }
        
        Write-Host -NoNewline "Running "
        Write-Host -ForegroundColor DarkGray "git init"
        git init | Out-Null

        Write-Host -NoNewline "Running "
        Write-Host -ForegroundColor DarkGray "git remote add origin ${info.GitURL}"
        git remote add origin $info.GitURL | Out-Null
    }
    finally {
        Pop-Location
    }

    if ($Open) {
        Open-Repo -Repo $Repo -Service $Service -Path $Path
    }

    return $info
}

function Open-Repo {
    param(
        [string]
        [Parameter(Mandatory = $true, ValueFromPipeline = $true, HelpMessage = "The name of the repository that you wish to synchronize (e.g. Namespace/RepoName)")]
        $Repo,

        [string]
        [Parameter(HelpMessage = "The service hosting your repository (e.g. github.com)")]
        [ValidateSet("github.com", "dev.azure.com", "gitlab.com", "bitbucket.org")]
        $Service = "github.com",

        [string]
        [Parameter(HelpMessage = "The directory within which your repositories will be checked out (e.g. /src/)")]
        $Path = $env:DEV_DIRECTORY
    )

    $info = Get-RepoInfo -Repo $Repo -Service $Service -Path $Path

    if (-not $info.Exists) {
        Write-Error -Category ResourceExists -Message "The repository $Service/$Repo does not exist." -RecommendedAction "Use New-Repo to create it."
        return
    }

    Set-Location -Path $info.Path

    return $info
}