# Installation Guide

## Downloading Git-Tool
We publish the latest Git-Tool releases on [GitHub][release] for all of our supported platforms.
Head on over and download the executable for your platform.

::: tip
 - **Windows** users on `x86_64` platforms should download `git-tool-windows-amd64.exe`.
 - **Linux** users on `x86_64` platforms should download `git-tool-linux-amd64`.
 - **MacOS** users on `x86_64` or `arm64` platforms should download `git-tool-darwin-amd64`.

 If your platform is not supported, please reach out by creating a [GitHub Issue](https://github.com/SierraSoftworks/git-tool/issues/new/choose)
 and we will try to help.
:::


Once you have downloaded the latest Git-Tool executable, rename it to `git-tool` and place it in a directory which is on your `$PATH`.
On Linux and MacOS machines, you may need to use `chmod +x git-tool` to mark the program as executable.

## Setting up your `PATH`

::: danger
Don't put your download folder on your `$PATH` - it's probably not a good idea from a security perspective.
Instead, find a different directory to store `git-tool` in, I personally use a dedicated **Programs** folder
which is used for exactly these kinds of tools.
:::

### Windows
To add or modify environment variables on Windows, you can press <kbd>Win</kbd>+<kbd>Pause</kbd> and then
choose **Advanced System Settings** &rarr; **Environment Variables**. This will open the Environment Variables
editor.

There are two changes you'll want to make:

1. Find the `Path` environment variable and edit it, adding the directory you saved Git-Tool into as the bottom of this list.
2. Add a new environment variable called `GITTOOL_CONFIG` and set it to `%USERPROFILE%\git-tool.yml` (or another location if you'd prefer).


When you're done, save your changes by clicking on **Ok**.

### Linux
The easiest way to modify your `PATH` on Linux is to open up your `~/.profile` file and add the following (fill in the path to the directory you placed
`git-tool` into earlier):

```bash
# ~/.profile

export PATH="$PATH:$HOME/apps" # Add ~/apps to your path
export GITTOOL_CONFIG="$HOME/.config/git-tool.yml" # Set ~/.config/git-tool.yml as your config
```

### MacOS
The easiest way to ensure that a program is on your path in MacOS is to simply drag and drop it into your **Applications** folder. To setup
the `GITTOOL_CONFIG` path, open up your `.bash_profile` file using `vi ~/.bash_profile` and add the following.

```bash
# ~/.bash_profile

# If you prefer to keep git-tool in a different folder, you can update your PATH
# export PATH="$PATH:$HOME/apps"

export GITTOOL_CONFIG="$HOME/Library/Preferences/git-tool.yml"
```

## Checkpoint #1
You should now be able to run `git-tool --version` and see something similar to the following appear in your terminal.

```
Git-Tool v2.2.0
```

::: warning
If you instead get an error saying that `git-tool` could not be found, that means that it is either not on your path or
not marked as executable. First try restarting your terminal to make sure you've got the latest `$PATH` loaded and if that
doesn't help, head on back to the [Setting up your PATH](#setting-up-your-path) section and check that you haven't missed anything.
:::

## Setup your Config
Now that you can run Git-Tool, the next step is to configure it. If you've followed the steps up to now,
you will have a `$GITTOOL_CONFIG` environment variable set. That means we can quickly open up your favourite
editor and start getting things configured.

:::: code-group
::: code-group-item Windows (PowerShell)
```powershell
notepad $env:GITTOOL_CONFIG
```
:::

::: code-group-item Windows (cmd)
```batch
notepad %GITTOOL_CONFIG%
```
:::

::: code-group-item Linux/MacOS
```bash
vi $GITTOOL_CONFIG
```
:::
::::

Drop in the following starter configuration and change the `directory` to point wherever you'd like to keep your repositories.

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
    command: powershell

features:
  #  Set this to false if you don't want to send crash information to us
  telemetry: true
```

## Checkpoint #2
Now that you've got your config added, let's make sure that `git-tool` can find it. Run the following command
and make sure that it prints out the same config you just saved.

```powershell
git-tool config
```

::: warning
If the config doesn't look like the one you just added, that means Git-Tool couldn't find it. Make sure that
you have configured the `$GITTOOL_CONFIG` environment variable to match the path to the file and that it is
readable.
:::

## Setting up your Shell
The last step in setting up Git-Tool is to configure your shell to support autocompletion and add the `gt` alias.

### PowerShell
To get the most out of Git-Tool (including adding the `gt` alias), you'll need to make changes
to your PowerShell profile, which is run whenever you start a new command prompt.

In an existing PowerShell terminal, open your profile file for modification using `notepad $PROFILE.CurrentUserAllHosts`
or `vi $PROFILE.CurrentUserAllHosts` and add the following to it:

```powershell
# Open this file with:
# notepad $PROFILE.CurrentUserAllHosts

# This adds an alias for Git-Tool so you can simply type "gt"
New-Alias -Name gt -Value git-tool

# This sets up autocomplete support for git-tool and "gt"
Invoke-Expression (&git-tool shell-init powershell)
```

### bash
To get the most out of Git-Tool (including adding the `gt` alias), you'll need to make changes
to your bash profile, which is run whenever you start a new terminal session.

```bash
# ~/.bashrc

alias gt="git-tool"
eval "$(git-tool shell-init bash)"
```

### zsh
To get the most out of Git-Tool (including adding the `gt` alias), you'll need to make changes
to your zsh profile, which is run whenever you start a new terminal session.

```bash
# ~/.zshrc
alias gt="git-tool"
eval "$(git-tool shell-init zsh)"
```

### fish
To get the most out of Git-Tool (including adding the `gt` alias), you'll need to make changes
to your fish config, which is run whenever you start a new terminal session.

```bash
# ~/config.fish
alias gt="git-tool"
complete -f -c git-tool -a "(git-tool complete)"
```


## Checkpoint #3
You should now be fully set up. Restart your terminal to load the newest version of your profile and
try running `gt --version`, you should see something similar to this appear. If you do, **Congratulations**,
you're all set up! :rocket:

```
Git-Tool v2.2.0
```


[release]: https://github.com/SierraSoftworks/git-tool/releases