# Git Tool
**Simplify checking out your Git repositories in a structured directory space**

Git Tool is a PowerShell module which simplifies managing your portfolio of repositories
by keeping them organized in a simple and concise directory structure inspired by Go.

## Installing the Module

```powershell
Install-Module -Name Git-Tools

# Choose which directory you wish to store your repositories in
Set-DevDirectory "C:\dev"
```

You can also modify your PowerShell profile to add these lines and automatically
have `Git-Tool` imported into your shell whenever it is needed.

```powershell
λ notepad $profile.CurrentUserAllHosts
```

## Using the Module

```powershell
C:\ PS> Get-Repo sierrasoftworks/git-tool

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

C:\ PS> Open-Repo sierrasoftworks/git-tool
C:\dev\github.com\SierraSoftworks\git-tool PS> Get-GitIgnore powershell | Set-Content .gitignore
```

## Commands

### Get-Repos

```
NAME
    Get-Repos
    
SYNOPSIS
    Gets a list of all the known repositories available for a certain hosting service.
    
    
SYNTAX
    Get-Repos [[-Service] <String>] [[-Path] <String>] [<CommonParameters>]
    
    
DESCRIPTION
    Lists all of the repositories for a given service for which you have local copies. These repositories
    may then be used to perform bulk operations, searches and more. In addition to this, the data reported
    by this command will be used for autocompletion of repository names in commands like Get-Repo, Get-RepoInfo,
    Open-Repo and more.
    

PARAMETERS
    -Service <String>
        The remote hosting provider for the repositories in question.
        
        Required?                    false
        Position?                    1
        Default value                $GitTool.Service
        Accept pipeline input?       false
        Accept wildcard characters?  false
        
    -Path <String>
        The directory within which all of your repositories are stored. Defaults to the value of $GitTool.Directory
        if not specified.
        
        Required?                    false
        Position?                    2
        Default value                $GitTool.Directory
        Accept pipeline input?       false
        Accept wildcard characters?  false
        
    <CommonParameters>
        This cmdlet supports the common parameters: Verbose, Debug,
        ErrorAction, ErrorVariable, WarningAction, WarningVariable,
        OutBuffer, PipelineVariable, and OutVariable. For more information, see 
        about_CommonParameters (https:/go.microsoft.com/fwlink/?LinkID=113216). 
    
INPUTS
    
OUTPUTS
    
    -------------------------- EXAMPLE 1 --------------------------
    
    C:\ PS>Get-Repos
    
    SierraSoftworks/bender
    SierraSoftworks/git-tool
    SierraSoftworks/iridium
    SierraSoftworks/sentry-go
    SierraSoftworks/vue-template
    
    
    
    
    
RELATED LINKS
    Get-Repo 
    Get-RepoInfo 
    Open-Repo 
```

### Get-RepoNamespaces

```
NAME
    Get-RepoNamespaces
    
SYNOPSIS
    Gets a list of all the known repository namespaces available for a certain hosting service.
    
    
SYNTAX
    Get-RepoNamespaces [[-Service] <String>] [[-Path] <String>] [<CommonParameters>]
    
    
DESCRIPTION
    Lists all of the namespaces for a given service for which you have local repositories. These
    namespaces form the first component of a repository name (i.e. namespace/name) and this information
    will primarily be used to provide autocompletion hints for the New-Repo command.
    

PARAMETERS
    -Service <String>
        The remote hosting provider for the repository namespaces in question.
        
        Required?                    false
        Position?                    1
        Default value                $GitTool.Service
        Accept pipeline input?       false
        Accept wildcard characters?  false
        
    -Path <String>
        The directory within which all of your repositories are stored. Defaults to the value of $GitTool.Directory
        if not specified.
        
        Required?                    false
        Position?                    2
        Default value                $GitTool.Directory
        Accept pipeline input?       false
        Accept wildcard characters?  false
        
    <CommonParameters>
        This cmdlet supports the common parameters: Verbose, Debug,
        ErrorAction, ErrorVariable, WarningAction, WarningVariable,
        OutBuffer, PipelineVariable, and OutVariable. For more information, see 
        about_CommonParameters (https:/go.microsoft.com/fwlink/?LinkID=113216). 
    
INPUTS
    
OUTPUTS
    
    -------------------------- EXAMPLE 1 --------------------------
    
    C:\ PS>Get-RepoNamespaces
    
    SierraSoftworks
    spartan563
    
    
    
    
    
RELATED LINKS
    New-Repo 
```

### Get-RepoInfo

```
NAME
    Get-RepoInfo
    
SYNOPSIS
    Gets an object describing the full details of a repo based on its name and hosting service.
    
    
SYNTAX
    Get-RepoInfo [-Repo] <String> [[-Service] <String>] [[-Path] <String>] [<CommonParameters>]
    
    
DESCRIPTION
    Generates an object which includes information about a repo including its expected location on
    your local filesystem, the webpage on which it may be viewed, the URL which should be used by
    a git clone operation and more.
    

PARAMETERS
    -Repo <String>
        The name of the repository which you would like to describe.
        
        Required?                    true
        Position?                    1
        Default value                
        Accept pipeline input?       true (ByValue)
        Accept wildcard characters?  false
        
    -Service <String>
        The remote hosting provider for the repository in question.
        
        Required?                    false
        Position?                    2
        Default value                $GitTool.Service
        Accept pipeline input?       false
        Accept wildcard characters?  false
        
    -Path <String>
        The directory within which all of your repositories are stored. Defaults to the value of $GitTool.Directory
        if not specified.
        
        Required?                    false
        Position?                    3
        Default value                $GitTool.Directory
        Accept pipeline input?       false
        Accept wildcard characters?  false
        
    <CommonParameters>
        This cmdlet supports the common parameters: Verbose, Debug,
        ErrorAction, ErrorVariable, WarningAction, WarningVariable,
        OutBuffer, PipelineVariable, and OutVariable. For more information, see 
        about_CommonParameters (https:/go.microsoft.com/fwlink/?LinkID=113216). 
    
INPUTS
    
OUTPUTS
    
    -------------------------- EXAMPLE 1 --------------------------
    
    C:\ PS>Get-RepoInfo sierrasoftworks/git-tool
    
    Name                           Value
    ----                           -----
    WebURL                         https://github.com/SierraSoftworks/git-tool
    Service                        github.com
    Repo                           SierraSoftworks/git-tool
    Path                           C:\dev\github.com\SierraSoftworks\git-tool
    Exists                         True
    GitURL                         git@github.com:SierraSoftworks/git-tool.git
    
    
    
    
    
RELATED LINKS
```

### Get-Repo

```
NAME
    Get-Repo
    
SYNOPSIS
    Gets the latest version of a particular repository from the upstream hosting provider.
    
    
SYNTAX
    Get-Repo [-Repo] <String> [[-Service] <String>] [[-Branch] <String>] [[-Path] <String>] [-Reconfigure] [-Open] [<CommonParameters>]
    
    
DESCRIPTION
    Used to quickly acquire the latest version of a given repository from a remote hosting provider.
    Will automatically execute a git clone operation if necessary, falling back to git pull if the
    repository already exists locally.
    

PARAMETERS
    -Repo <String>
        The name of the repository which you would like to fetch.
        
        Required?                    true
        Position?                    1
        Default value                
        Accept pipeline input?       true (ByValue)
        Accept wildcard characters?  false
        
    -Service <String>
        The remote hosting provider from which you would like to fetch the repository.
        
        Required?                    false
        Position?                    2
        Default value                $GitTool.Service
        Accept pipeline input?       false
        Accept wildcard characters?  false
        
    -Branch <String>
        The repository branch which you owuld like to checkout.
        
        Required?                    false
        Position?                    3
        Default value                
        Accept pipeline input?       false
        Accept wildcard characters?  false
        
    -Path <String>
        The directory within which all of your repositories are stored. Defaults to the value of $GitTool.Directory
        if not specified.
        
        Required?                    false
        Position?                    4
        Default value                $GitTool.Directory
        Accept pipeline input?       false
        Accept wildcard characters?  false
        
    -Reconfigure [<SwitchParameter>]
        Whether you would like to reconfigure the local repository to use the updated remote address in situations where that has changed.
        
        Required?                    false
        Position?                    named
        Default value                False
        Accept pipeline input?       false
        Accept wildcard characters?  false
        
    -Open [<SwitchParameter>]
        Whether or not to open the repository once it has been fetched.
        
        Required?                    false
        Position?                    named
        Default value                False
        Accept pipeline input?       false
        Accept wildcard characters?  false
        
    <CommonParameters>
        This cmdlet supports the common parameters: Verbose, Debug,
        ErrorAction, ErrorVariable, WarningAction, WarningVariable,
        OutBuffer, PipelineVariable, and OutVariable. For more information, see 
        about_CommonParameters (https:/go.microsoft.com/fwlink/?LinkID=113216). 
    
INPUTS
    
OUTPUTS
    
    -------------------------- EXAMPLE 1 --------------------------
    
    PS C:\>Get-Repo sierrasoftworks/git-tool
    
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
    
    
    
    
    
RELATED LINKS
```

### Get-CurrentRepo

```
NAME
    Get-CurrentRepo
    
SYNOPSIS
    Gets the information describing the repository at your current location.
    
    
SYNTAX
    Get-CurrentRepo [[-Path] <String>] [<CommonParameters>]
    
    
DESCRIPTION
    Considers your current location/directory and determines whether it is a valid repository.
    If it is, this will retrieve the repository information describing this repo.
    

PARAMETERS
    -Path <String>
        The directory within which all of your repositories are stored. Defaults to the value of $GitTool.Directory
        if not specified.
        
        Required?                    false
        Position?                    1
        Default value                $GitTool.Directory
        Accept pipeline input?       false
        Accept wildcard characters?  false
        
    <CommonParameters>
        This cmdlet supports the common parameters: Verbose, Debug,
        ErrorAction, ErrorVariable, WarningAction, WarningVariable,
        OutBuffer, PipelineVariable, and OutVariable. For more information, see 
        about_CommonParameters (https:/go.microsoft.com/fwlink/?LinkID=113216). 
    
INPUTS
    
OUTPUTS
    
    -------------------------- EXAMPLE 1 --------------------------
    
    C:\ PS>Get-CurrentRepo
    
    Name                           Value
    ----                           -----
    WebURL                         https://github.com/SierraSoftworks/git-tool
    Service                        github.com
    Repo                           SierraSoftworks/git-tool
    Path                           C:\dev\github.com\SierraSoftworks\git-tool
    Exists                         True
    GitURL                         git@github.com:SierraSoftworks/git-tool.git
    
    
    
    
    
RELATED LINKS
    Get-RepoInfo 
```

### Get-GitIgnore

```
NAME
    Get-GitIgnore

SYNOPSIS
    Fetches a gitignore file from gitignore.io for the given language.


SYNTAX
    Get-GitIgnore [[-Language] <String>] [<CommonParameters>]


DESCRIPTION
    Uses the gitignore.io API to download the contents of a gitignore file
    which provides sensible defaults for the given language. Can be used to quickly
    bootstrap a repository or add a gitignore file to an existing one.


PARAMETERS
    -Language <String>
        The programming language or tool name for which a gitignore file will be retrieved.

        Required?                    false
        Position?                    1
        Default value                $GitTool.GitIgnore.Default
        Accept pipeline input?       false
        Accept wildcard characters?  false

    <CommonParameters>
        This cmdlet supports the common parameters: Verbose, Debug,
        ErrorAction, ErrorVariable, WarningAction, WarningVariable,
        OutBuffer, PipelineVariable, and OutVariable. For more information, see
        about_CommonParameters (https:/go.microsoft.com/fwlink/?LinkID=113216).

INPUTS

OUTPUTS

    -------------------------- EXAMPLE 1 --------------------------

    PS C:\>Get-GitIgnore powershell | Set-Content .gitignore







RELATED LINKS
    https://gitignore.io
```

### New-Repo

```
NAME
    New-Repo

SYNOPSIS
    Creates a new local repository with the provided name.


SYNTAX
    New-Repo [-Repo] <String> [[-Service] <String>] [[-Path] <String>] [-Open] [[-GitIgnore] <String>] [<CommonParameters>]


DESCRIPTION
    Used to quickly bootstrap a local repository by automatically creating the correct
    folder structure, configuring your Git remotes and optionally adding a relevant gitignore
    file. Upon completion, you may switch to the location of the newly created repo by either
    passing the -Open flag or by using Open-Repo.


PARAMETERS
    -Repo <String>
        The name of the repository which you would like to fetch.

        Required?                    true
        Position?                    1
        Default value
        Accept pipeline input?       true (ByValue)
        Accept wildcard characters?  false

    -Service <String>
        The remote hosting provider from which you would like to fetch the repository.

        Required?                    false
        Position?                    2
        Default value                github.com
        Accept pipeline input?       false
        Accept wildcard characters?  false

    -Path <String>
        The directory within which all of your repositories are stored. Defaults to the value of $GitTool.Directory
        if not specified.

        Required?                    false
        Position?                    3
        Default value                $GitTool.Directory
        Accept pipeline input?       false
        Accept wildcard characters?  false

    -Open [<SwitchParameter>]
        Whether you would like to reconfigure the local repository to use the updated remote address in situations where that has changed.

        Required?                    false
        Position?                    named
        Default value                False
        Accept pipeline input?       false
        Accept wildcard characters?  false

    -GitIgnore <String>
        The language or tool for which you would like to fetch a gitignore file. If not provided, defaults to your globally configured
        $GitTool.GitIgnore.Default option.

        Required?                    false
        Position?                    4
        Default value                $GitTool.GitIgnore.Default
        Accept pipeline input?       false
        Accept wildcard characters?  false

    <CommonParameters>
        This cmdlet supports the common parameters: Verbose, Debug,
        ErrorAction, ErrorVariable, WarningAction, WarningVariable,
        OutBuffer, PipelineVariable, and OutVariable. For more information, see
        about_CommonParameters (https:/go.microsoft.com/fwlink/?LinkID=113216).

INPUTS

OUTPUTS

    -------------------------- EXAMPLE 1 --------------------------

    PS C:\>New-Repo sierrasoftworks/git-tool

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




    -------------------------- EXAMPLE 2 --------------------------

    PS C:\>New-Repo sierrasoftworks/git-tool -GitIgnore powershell

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





RELATED LINKS
```

### Open-Repo

```
NAME
    Open-Repo

SYNOPSIS
    Changes your location to the directory of the repository you have specified.


SYNTAX
    Open-Repo [-Repo] <String> [[-Service] <String>] [[-Path] <String>] [<CommonParameters>]


DESCRIPTION
    Quickly changes the location of your session to the directory holding the
    repository you have specified, if it exists.


PARAMETERS
    -Repo <String>
        The name of the repository which you would like to open.

        Required?                    true
        Position?                    1
        Default value
        Accept pipeline input?       true (ByValue)
        Accept wildcard characters?  false

    -Service <String>
        The remote hosting provider from which you would like to open the repository.

        Required?                    false
        Position?                    2
        Default value                github.com
        Accept pipeline input?       false
        Accept wildcard characters?  false

    -Path <String>
        The directory within which all of your repositories are stored. Defaults to the value of $GitTool.Directory
        if not specified.

        Required?                    false
        Position?                    3
        Default value                $GitTool.Directory
        Accept pipeline input?       false
        Accept wildcard characters?  false

    <CommonParameters>
        This cmdlet supports the common parameters: Verbose, Debug,
        ErrorAction, ErrorVariable, WarningAction, WarningVariable,
        OutBuffer, PipelineVariable, and OutVariable. For more information, see
        about_CommonParameters (https:/go.microsoft.com/fwlink/?LinkID=113216).

INPUTS

OUTPUTS

    -------------------------- EXAMPLE 1 --------------------------

    PS C:\>Open-Repo sierrasoftworks/git-tool

    Name                           Value
    ----                           -----
    WebURL                         https://github.com/SierraSoftworks/git-tool
    Service                        github.com
    Repo                           SierraSoftworks/git-tool
    Path                           C:\dev\github.com\SierraSoftworks\git-tool
    Exists                         True
    GitURL                         git@github.com:SierraSoftworks/git-tool.git





RELATED LINKS
```