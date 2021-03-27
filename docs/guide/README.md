# Getting Started
Welcome to Git-Tool, we hope you'll find it to be an awesome addition to your toolkit
and that it'll bring you plenty of smiles. There are a few things you'll need to do
to get yourself set up and working and these are covered in each of the **Getting
Started** guides you'll find here.

::: tip
This guide is the high-level run-through for anyone familiar with setting up Git-Tool
or who likes guessing their way to a conclusion. If you run into trouble or want something
more detailed, use the links in each **Step**.
:::

#### Step #1: [Installation](installation.md)
You can download the latest version of Git-Tool from our [GitHub releases][release] page.
Pop it into your `$PATH`, setup a [config](../config/README.md) file,
[configure your shell](installation.md#setting-up-your-shell) and you're good to go!

:::: code-group
::: code-group-item PowerShell
```powershell
# $PROFILE.CurrentUserAllHosts

$env:GITTOOL_CONFIG="${env:HOME}/git-tool.yml"

# This adds an alias for Git-Tool so you can simply type "gt"
New-Alias -Name gt -Value git-tool

# This sets up autocomplete support for git-tool and "gt"
Invoke-Expression (&git-tool shell-init powershell)
```
:::

::: code-group-item bash
```bash
# ~/.bashrc

export GITTOOL_CONFIG="$HOME/.config/git-tool.yml"

alias gt="git-tool"
eval "$(git-tool shell-init bash)"
```
:::

::: code-group-item zsh
```bash
# ~/.zshrc

export GITTOOL_CONFIG="$HOME/Library/Preferences/git-tool.yml"

alias gt="git-tool"
eval "$(git-tool shell-init zsh)"
```
:::
::::

#### Step #2: [Configuration](../config/README.md)
Setup your `$GITTOOL_CONFIG` file with the repository hosting services and apps you want
to use, point it at your development directory and Git-Tool will do the rest.

```yaml
---
directory: "C:\\dev" # CHANGE ME
services:
  - domain: github.com
    website: "https://{{ .Service.Domain }}/{{ .Repo.FullName }}"
    httpUrl: "https://{{ .Service.Domain }}/{{ .Repo.FullName }}.git"
    gitUrl: "git@{{ .Service.Domain }}:{{ .Repo.FullName }}.git"
    default: true
    pattern: "*/*"
apps:
  - name: shell
    command: pwsh # CHANGE ME

features:
  #  Set this to false if you don't want to send crash information to us
  telemetry: true
```

#### Step #3: [Linking to GitHub](github.md)
Git-Tool <3 GitHub and can automatically create repositories there whenever you run `gt new`.

To set this up, generate a [new Personal Access Token](https://github.com/settings/tokens/new?scopes=repo)
with the `repo` scope and run the following command to store it in your local keychain.

```powershell
gt auth github.com
```

#### Bonus Step: [Updating Git-Tool](updates.md)
We update Git-Tool regularly to patch bugs, add features and ensure that any potential
security vulnerabilities in Git-Tool or its dependencies are closed as quickly as possible.

You'll find our list of releases on [GitHub][release] and can subscribe there for notifications,
but the quickest way to update is to run `gt update`.

::: warning
Due to the way Git-Tool runs and updates itself, you'll need to make sure that you close down any shells
it has launched before running `gt update`.
:::

[release]: https://github.com/SierraSoftworks/git-tool/releases
[new-issue]: https://github.com/SierraSoftworks/git-tool/issues/new/choose