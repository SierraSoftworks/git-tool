# Git Tool
**Simplify checking out your Git repositories in a structured directory space**

Git Tool is a powerful tool for managing your Git repositories, storing them in
a consistent folder structure and simplifying access when you need it.

## Features
 - **Quickly open repositories** whether they are already cloned locally or not, using your favourite Git services and a concise folder structure.
 - **Launch applications** within the context of your repositories quickly and consistently.
 - **Weekly scratchpads** to help organize random work and doodles with minimal effort.
 - **Aliases** to make opening your most common repositories as quick as possible.
 - **Fast autocompletion** on all platforms with support for "sequence search" (`ssgt` matches `SierraSoftworks/git-tool`) as found in Sublime and VSCode.

## Example

```powershell
# Open the sierrasoftworks/git-tool repo in your default app (bash by default)
# This will clone the repo automatically if you don't have it yet.
gt o sierrasoftworks/git-tool

# Open the github.com/sierrasoftworks/git-tool repo in VS Code (if listed in your config)
gt o code github.com/sierrasoftworks/git-tool

# Create a new repository and instruct GitHub to create the repo as well, if you
# have permission to do so.
gt new github.com/sierrasoftworks/demo-repo

# Show info about the repository in your current directory
gt i

# Show information about a specific repository
gt i dev.azure.com/sierrasoftworks/opensource/git-tool

# Open your shell in the current week's scratch directory
gt s
```

## Installation

#### Step 1: Download the latest Release
Make sure you download the latest [release][] for your platform and place it in a directory on your `$PATH`.

#### Step 2: Ensure that you can run `git-tool`

```
λ git-tool --version
gt version 1.2.13+1
```

#### Step 3: Configure your Installation
Add a `git-config.yml` file somewhere and fill it in with the following (modifying your directory to match your chosen development folder).

```yaml
---
directory: /home/bpannell/dev
services:
  - domain: github.com
    website: "https://{{ .Service.Domain }}/{{ .Repo.FullName }}"
    httpUrl: "https://{{ .Service.Domain }}/{{ .Repo.FullName }}.git"
    gitUrl: "git@{{ .Service.Domain }}:{{ .Repo.FullName }}.git"
    default: true
    pattern: "*/*"
apps:
  - name: shell
    command: bash
    default: true
  - name: code
    command: code
    args:
      - .
```

Then update your environment to inform `git-tool` of your config file. While you're at it, enable autocomplete.

##### Windows
```powershell
notepad $PROFILE.CurrentUserAllHosts
```

Then add the following and save.

```powershell
# The path to your git-tool config file.
$env:GITTOOL_CONFIG = "C:\dev\git-tool.yml"

# This adds an alias for Git-Tool so you can simply type "gt"
New-Alias -Name gt -Value "git-tool.exe"

# This sets up autocomplete support for git-tool and "gt"
Invoke-Expression (&git-tool shell-init powershell)
```

##### Linux
```bash
vi ~/.bashrc
```

Then add the following:

```bash
# ~/.bashrc
alias gt="git-tool"
eval "$(git-ignore shell-init bash)"
```

##### MacOS
```zsh
vi ~/.zshrc
```

Then add the following:

```zsh
# ~/.zshrc
alias gt="git-tool"
eval "$(git-ignore shell-init zsh)"
```

## Adding new Services
Git Tool has been written to support a wide range of Git servers and allows you to add your own via the config file.

#### Azure DevOps
```yaml
services:
  - domain: dev.azure.com
    website: "https://{{ .Service.Domain }}/{{ .Repo.Namespace }}/_git/{{ .Repo.Name }}"
    httpUrl: "https://{{ .Service.Domain }}/{{ .Repo.Namespace }}/_git/{{ .Repo.Name }}"
    gitUrl: "git@ssh.{{ .Service.Domain }}:v3/{{ .Repo.FullName }}"
    pattern: "*/*/*"
```

#### BitBucket
```yaml
services:
  - domain: bitbucket.org
    website: "https://{{ .Service.Domain }}/{{ .Repo.FullName }}"
    httpUrl: "https://{{ .Service.Domain }}/{{ .Repo.FullName }}.git"
    gitUrl: "git@{{ .Service.Domain }}:{{ .Repo.FullName }}.git"
    pattern: "*/*"
```

#### GitLab
```yaml
services:
  - domain: gitlab.com
    website: "https://{{ .Service.Domain }}/{{ .Repo.FullName }}"
    httpUrl: "https://{{ .Service.Domain }}/{{ .Repo.FullName }}.git"
    gitUrl: "git@{{ .Service.Domain }}:{{ .Repo.FullName }}.git"
    pattern: "*/*"
```

#### Custom
When adding a custom service, you will need to ensure that you provide the various templates necessary for generating URLs
as well as the glob `pattern` which will be used to identify repositories within the service's development directory. In
the case of most Git services, this will be `*/*` (corresponding to the organization name and repository name); however some
services like Azure DevOps make use of different patterns.

## Adding new Apps
Git Tool has the ability to launch applications within the context of your repositories. This is useful when you want to
quickly open a shell or your favourite editor and start working, however you can also add a wide range of other applications
there. Here are a few examples.

#### Admin PowerShell on Windows
```yaml
apps:
  - name: admin
    command: powershell.exe
    args:
      - "Start-Process"
      - "powershell.exe"
      - "-Verb runas"
      - "-ArgumentList"
      - "@('-NoExit', '-Command', 'cd ''{{ .Target.Path }}''')"
```

#### Windows Explorer
```yaml
apps:
  - name: explorer
    command: explorer.exe
    args:
      - .
```

## Aliases
For your most common repositories, it can often make sense to give distinct aliases. These aliases allow you to quickly and
exactly specify a repository without typing its full name or relying on autocomplete.

```yaml
aliases:
    blog: github.com/sierrasoftworks/blog
```

You can use an alias anywhere you would specify a repository name, such as `gt o blog`.

[release]: https://github.com/SierraSoftworks/git-tool/releases
