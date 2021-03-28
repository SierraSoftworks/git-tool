# Setup

## update <Badge text="v1.4+"/>
We like to think that we're pretty good at updating Git-Tool to ensure that
it is using the latest stable libraries and includes all the best features.
Of course, that also means some pretty regular releases (usually once every
week or two) and for a command line application, that can mean that things
get outdated pretty quickly.

To help with that, we added the `gt update` command which will automatically
download the latest version of Git-Tool for your operating system (if we
support it in our release builds).

::: warning
We use an [three-phase update strategy][update-strategy] for Git-Tool and,
as a result, we can't update it if other instances are currently running.
**To make sure the update completes successfully, close down all running
instances of Git-Tool (including shells it has launched) before updating.**
:::

#### Example
```powershell
# Update to the latest available release for your OS
gt update

# Update to a specific release version
gt update v2.2.0
```

## shell-init <Badge text="v1.5+"/>
The `gt shell-init` command is part of the magic that provides cross-platform
autocomplete suggestions and lets us upgrade our infrastructure for that
automatically.

It is responsible for generating a runnable shell command which initializes
everything needed by Git-Tool for your environment.

::: tip
Take a look at the [shell setup guide](../guide/shell-init.md) for detailed
instructions on setting up your shell environment to get the most out of
Git-Tool.
:::

#### Example
:::: code-group
::: code-group-item PowerShell
```powershell
# $PROFILE.CurrentUserAllHosts
Invoke-Expression (&git-tool shell-init powershell)
```
:::

::: code-group-item bash
```bash
# ~/.bashrc
eval "$(git-tool shell-init bash)"
```
:::

::: code-group-item zsh
```bash
# ~/.zshrc
eval "$(git-tool shell-init zsh)"
```
:::

::: code-group-item fish
```bash
# ~/config.fish
complete -f -c git-tool -a "(git-tool complete)"
```
:::
::::

## complete <Badge text="v1.2+"/> <Badge text="internal" type="warning"/>
The `gt complete` command is part of the internal autocomplete plumbing
used by Git-Tool. It is a flagrant copy of the
[dotnet CLI autocomplete](https://docs.microsoft.com/en-us/dotnet/core/tools/enable-tab-autocomplete)
interface and accepts a combination of the current command input and the
position of the cursor (if provided) to generate suggestions.

::: warning
This command is part of Git-Tool's internal API and may change without notice and without
a major version bump. Please avoid depending on its semantics and instead use `gt shell-init`
to configure your environment.
:::

#### Options
 - `--position` allows a shell which supports this parameter to provide information about the current cursor position to improve autocomplete suggestion quality.

#### Example
```powershell
gt complete "gt o git-too"
```

[update-strategy]: https://blog.sierrasoftworks.com/2019/10/15/app-updates/