---
description: Here are the commands you can use to manage repositories with Git-Tool.
---

# Repositories

Git-Tool's main purpose is managing your local development directory, ensuring that your repositories are kept organized and are available when you need them with a minimum amount of cognitive effort on your part.

### Directory Structure

Git-Tool uses a directory structure very similar to `GOPATH`. If you're curious why we've chosen this approach, please read my [blog post](https://blog.sierrasoftworks.com/2019/04/15/git-tool/#background) on the topic. If you are simply curious what that means, here's an example:

```text
.
└── dev/
    └── github.com/
        ├── notheotherben/
        │   └── cv/
        └── sierrasoftworks/
            ├── bender/
            ├── blog/
            ├── git-tool/
            ├── iridium/
            └── vue-template/
```

## open

The first place you're likely to start with Git-Tool is opening a repo you want to work on. To do so, you'll use the `gt open` command, which allows you to launch a shell \(or any other app you have defined in your [config](../config/overview.md)\) inside that repository's directory.

{% hint style="success" %}
Aliases take precedence over repos, which take precedence over apps. _When specifying an app, it should appear before the repo/alias parameter to avoid confusion._
{% endhint %}

New applications can be configured either by making changes to your configuration, or by using the [`git-tool config add`](config.md#config-add) command to install them from the GitHub registry. For example, you can use `gtconfig add apps/bash` to configure `bash` as an available app.

### Aliases

* `gt open`
* `gt o`
* `gt run`

### Options

* `-c`/`--create`  will create a new repository if one with this name doesn't exist locally.
* `-R`/`--no-create-remote`  will disable the [creation of a remote repository](../config/features.md#create_remote) when run with `-c`.

### Example

```bash
# Open a repository in your default application
gt o github.com/SierraSoftworks/git-tool

# Open a Visual Studio shell in the current repository
gt o vs

# Open a repository in VS Code
gt o github.com/SierraSoftworks/git-tool code
```

{% hint style="info" %}
If you are already inside a repository, you can specify only an app and it will launch in the context of the current repo, like `gt o vs` in the example above. _This can be very useful if the command you wish to run is on the complex end of the spectrum \(like launching a Visual Studio developer console\)._
{% endhint %}

## new

There's nothing new under the sun, but sometimes we like to build it anyway. If you're starting something new and want a fresh repo for it, the `gt new` command is your best friend. It will create and `git init` your repo, setup your remote hosting provider \(if supported\).

Of course, this command has auto-completion support and will suggest valid namespaces for your repository to appear in \(such as `github.com/notheotherben/` and `github.com/SierraSoftworks/`\), helping you quickly figure out where your repo should be created.

### Aliases

* `gt new`
* `gt n`
* `gt create`

### Options

* `-R`/`--no-create-remote`  will disable the [creation of a remote repository](../config/features.md#create_remote).
* `-o`/`--open`  will open this repository in your default application once it has been created. You can make this behaviour the default with the [`open_new_repo_in_default_app`](../config/features.md#open_new_repo_in_default_app) feature flag.

### Example

```bash
# Create a new repository
gt n github.com/notheotherben/demo

# Create (and open) a new repository
gt n --open github.com/notheotherben/demo

# Create a new repository but don't create it remotely
gt n --no-create-remote github.com/notheotherben/demo
```

## list

If you're trying to get a list of your repositories, Git-Tool has you covered. The `gt list` command will show you all of your locally cloned repositories and can be a useful tool if you need to \(for example\) write a script which performs a task across all of them.

{% hint style="info" %}
If you are migrating machines and want to clone your repositories, you can dump them with [`gt list -q`](repos.md#list) and then use [`gt clone`](repos.md#clone) to import them.
{% endhint %}

### Aliases

* `gt list`
* `gt ls`
* `gt ll`

### Options

* `-q`/`--quiet` will limit the output to only the repository's name. This output is useful for consumption by scripts.
* `--full` will print out a series of YAML documents, using `---` document separators, which contain detailed information about each of your repositories.

### Example

```bash
# List your repositories (and their web addresses)
gt ls

# List the repository names containing notheotherben
gt ls -q notheotherben

# Gather detailed information about sierralib repositories
gt ls --full github.com/SierraSoftworks/sierralib
```

## info

If you want to get access to some of the detailed information about a repository managed by Git-Tool, including things like the URLs associated with it or the path to the repo, you can use the `gt info` command.

### Aliases

* `gt info`
* `gt i`

### Example

```text
# Get information about the current repository
gt i

# Get information about a specific repository
gt i sierrasoftworks/git-tool
```

{% hint style="success" %}
You can omit the repository name if you want to get information about your current repo.
{% endhint %}

## clone

The `gt clone` command does everything the `gt open` command does, except open an application. If you're trying to quickly clone a list of repositories, such as when you're setting up your new dev-box, this is the command for you.

### Example

```bash
# Clone a repository into the appropriate folder
gt clone github.com/SierraSoftworks/git-tool
```

## fix

Git-Tool usually takes care of setting up your git `origin` remote, however sometimes you want to rename projects or even entire organizations. To make your life a little bit easier, the `gt fix` command will update your git `origin` remote to reflect the current repo information shown in [`gt info`](repos.md#info) \(which is based on its filesystem path\).

### Options

* `--all` will fix any repositories which match the provided pattern.

### Example

```bash
# Fix the git remote configuration for a single repository
gt fix github.com/SierraSoftworks/git-tool

# Fix the git remote configuration for a group of repositories
gt fix --all github.com/SierraSoftworks/
```

{% hint style="success" %}
The quickest way to update a repo's `origin` is to `mv $REPO $NEW_REPO` and then run `gt fix $NEW_REPO`.
{% endhint %}
