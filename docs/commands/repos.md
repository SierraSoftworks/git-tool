# Repositories

Git-Tool's main purpose is managing your local development directory, ensuring
that your repositories are kept organized and are available when you need them
with a minimum amount of cognitive effort on your part.

#### Directory Structure

Git-Tool uses a directory structure very similar to `GOPATH`. If you're curious
why we've chosen this approach, please read
my [blog post](https://blog.sierrasoftworks.com/2019/04/15/git-tool/#background)
on the topic. If you are simply curious what that means, here's an example:

<FileTree>

- dev
    - github.com
        - notheotherben
            - cv
        - sierrasoftworks
            - bender
            - blog
            - git-tool
            - iridium
            - vue-template

</FileTree>

## open <Badge text="v1.0+"/>

The first place you're likely to start with Git-Tool is opening a repo you want to work on.
To do so, you'll use the `gt open` command, which allows you to launch a shell (or any other
app you have defined in your [config](../config/README.md)) inside that repository's directory.

::: warning
Aliases take precedence over repos, which take precedence over apps. *When specifying an app,
it should appear before the repo/alias parameter to avoid confusion.*
:::

New applications can be configured either by making changes to your configuration, or by using the
[`git-tool config add`](config.md#apps) command to install them from the GitHub registry. For example, you
can use `gtconfig add apps/bash` to configure `bash` as an available app.

#### Aliases

- `gt open`
- `gt o`
- `gt run`

#### Options

- `-c`/`--create` <Badge text="v2.1+"/> will create a new repository if one with this name doesn't exist locally.
- `-R`/`--no-create-remote` <Badge text="v2.1+"/> will disable
  the [creation of a remote repository](../config/features.md#create-remote) when run with `-c`.

#### Example

```powershell
# Open a repository in your default application
gt o gh:SierraSoftworks/git-tool

# Open a Visual Studio shell in the current repository
gt o vs

# Open a repository in VS Code
gt o gh:SierraSoftworks/git-tool code
```

::: tip
If you are already inside a repository, you can specify only an app and it will launch in the
context of the current repo, like `gt o vs` in the example above. *This can be very useful if
the command you wish to run is on the complex end of the spectrum (like launching a Visual
Studio developer console).*
:::

## new <Badge text="v1.0+"/>

There's nothing new under the sun, but sometimes we like to build it anyway. If you're starting
something new and want a fresh repo for it, the `gt new` command is your best friend. It will
create and `git init` your repo, setup your remote hosting provider (if supported).

Of course, this command has auto-completion support and will suggest valid namespaces for your
repository to appear in (such as `github.com/notheotherben/` and `github.com/SierraSoftworks/`),
helping you quickly figure out where your repo should be created.

#### Aliases

- `gt new`
- `gt n`
- `gt create`

#### Options

- `-R`/`--no-create-remote` <Badge text="v2.1+"/> will disable
  the [creation of a remote repository](../config/features.md#create-remote).
- `-E`/`--no-check-exists` <Badge text="v3.3+"/> will
  disable [checking if the repository already exists](../config/features.md#check-exists).
- `-o`/`--open` <Badge text="v2.1+"/> will open this repository in your default application once it has been created.
  You can make this behaviour the default with the [
  `open_new_repo_in_default_app`](../config/features.md#open-new-repo-in-default-app) feature flag.
- `-f`/`--fork`/`--from` <Badge text="v3.9+"/> create a fork of an existing remote (on supported services) or a copy of
  an existing remote repository (on unsupported services)

#### Example

```powershell
# Create a new repository
gt n gh:notheotherben/demo

# Create (and open) a new repository
gt n --open gh:notheotherben/demo

# Create a new repository but don't create it remotely
gt n --no-create-remote gh:notheotherben/demo

# Fork a repository
gt n gh:notheotherben/demo --fork gh:git-fixtures/basic
```

## list <Badge text="v1.0+"/>

If you're trying to get a list of your repositories, Git-Tool has you covered. The `gt list`
command will show you all of your locally cloned repositories and can be a useful tool if you
need to (for example) write a script which performs a task across all of them.

::: tip
If you are migrating machines and want to clone your repositories, you can dump them with
`gt list -q` and then use `gt clone` to import them.
:::

#### Aliases

- `gt list`
- `gt ls`
- `gt ll`

#### Options

- `-q`/`--quiet` will limit the output to only the repository's name. This output is useful for consumption by scripts.
- `--full` will print out a series of YAML documents, using `---` document separators, which contain detailed
  information about each of your repositories.

#### Example

```powershell
# List your repositories (and their web addresses)
gt ls

# List the repository names containing notheotherben
gt ls -q notheotherben

# Gather detailed information about sierralib repositories
gt ls --full gh:SierraSoftworks/sierralib
```

## info <Badge text="v1.0+"/>

If you want to get access to some of the detailed information about a repository managed by Git-Tool,
including things like the URLs associated with it or the path to the repo, you can use the `gt info`
command.

#### Aliases

- `gt info`
- `gt i`

#### Example

```powershell
# Get information about the current repository
gt i

# Get information about a specific repository
gt i sierrasoftworks/git-tool
```

::: tip
You can omit the repository name if you want to get information about your current repo.
:::

## clone <Badge text="v2.1.19+"/>

The `gt clone` command does everything the `gt open` command does, except open an application.
If you're trying to quickly clone a list of repositories, such as when you're setting up your
new dev-box, this is the command for you.

#### Example

```powershell
# Clone a repository into the appropriate folder
gt clone gh:SierraSoftworks/git-tool

# Clone a series of repositories into the appropriate folders
# The repositories.txt file should contain a list of repositories, one per line
# e.g.
#   gh:SierraSoftworks/git-tool
#   gh:SierraSoftworks/tailscale-udm
#   gh:SierraSoftworks/vue-template
gt clone @repositories.txt
```

::: tip
As of <Badge text="v3.7.0+"/>, you can use the `@` symbol to specify a file
containing a list of repositories to clone. This can be paired with
[`gt list -q`](#list) to quickly backup and restore your list of local repositories,
or setup a new machine.
:::

## fix <Badge text="v2.1.4+"/>

Git-Tool usually takes care of setting up your git `origin` remote, however sometimes you
want to rename projects or even entire organizations. To make your life a little bit easier,
the `gt fix` command will update your git `origin` remote to reflect the current repo information
shown in [`gt info`](#info) (which is based on its filesystem path).

#### Options

- `--all` will fix any repositories which match the provided pattern.

#### Example

```powershell
# Fix the git remote configuration for a single repository
gt fix gh:SierraSoftworks/git-tool

# Fix the git remote configuration for a group of repositories
gt fix --all gh:SierraSoftworks/
```

::: tip
The quickest way to update a repo's name is to use the [`gt rename`](#rename) command
which will update the `origin` in addition to renaming/transferring the corresponding
GitHub repository.
:::

## remove <Badge text="v2.2.13+"/>

The `gt remove` command will remove a repository from your local machine. This is particularly
useful since a repository you have opened with [`gt open`](#open) will often have its shell spawned
in the directory you are attempting to delete.

#### Example

```powershell
# Remove a repository
gt remove gh:SierraSoftworks/git-tool
```

## rename <Badge text="v3.8.0+"/>

The `gt rename` command will rename a repository, including moving of the local directory and
updating the repository name (or transferring it to a different organization) on GitHub in cases
where both the source and target repository name are hosted on the same `GitHub/v3` service entry.

#### Aliases

- `gt mv`

#### Options

- `--no-move-remote` will cause the command to skip renaming/transferring the GitHub repository.

#### Example

```powershell
# Rename a repository in the same organization
gt rename gh:SierraSoftworks/git-tool gh:SierraSoftworks/gt

# Transfer a repository to a different organization
gt rename gh:SierraSoftworks/git-tool gh:SpechtLabs/git-tool

# Perform a local-only move from one service to another
gt rename gh:SierraSoftworks/git-tool ghp:SierraSoftworks/git-tool
```

::: note

In case you have a directory layout like this:

<FileTree>

- dev
    - github.com
        - SpechtLabs
            - git-tool

</FileTree>

but multiple remotes configured in the `git-tool` repository:

- `origin git@github.com:SpechtLabs/git-tool.git`
- `upstream git@github.com:SierraSoftworks/git-tool.git`

the `gt rename` command will only update the `origin` remote's URL to match the new repository name.
In this example, `gt rename gh:SpechtLabs/git-tool gh:SpechtLabs/gt` will result in the following remote
configuration:

- `origin git@github.com:SpechtLabs/gt.git`
- `upstream git@github.com:SierraSoftworks/git-tool.git`

:::
