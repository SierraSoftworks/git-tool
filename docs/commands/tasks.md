# Tasks <Badge text="v3.11+"/>
Many repositories come with a set of common operations you find yourself running over and over
again: building the project, running its tests, starting a development server, and so on. Git-Tool
lets each repository describe these operations as named **tasks** in a `git-tool.yml` file at its
root, and provides the `gt task` command to run them from within the repository.

Because tasks run commands defined by a repository, Git-Tool will only execute them once you have
confirmed that you **trust** the repository's configuration. This page covers the `git-tool.yml`
format, the `gt task` command, and how trust is managed with `gt trust`.

## The `git-tool.yml` file
A repository opts in to tasks (and worktree automation) by adding a `git-tool.yml` file to its
root. It looks like this:

```yaml
tasks:
  build:
    command: cargo
    args:
      - build
  test:
    command: cargo
    args:
      - test
    environment:
      - RUST_LOG=debug

worktree:
  symlinks:
    - node_modules
    - target
  tasks:
    - build
```

Each task mirrors the structure of an [app](../config/apps.md): it has a `command`, an optional list
of `args`, and an optional list of `environment` variables (in `KEY=value` form). Tasks are launched
using the same engine as your apps, so they benefit from the same
[templating](../config/templates.md) and signal forwarding.

## task <Badge text="v3.11+"/>
The `gt task` command runs a task defined in the current repository's `git-tool.yml` file. Run it
from within a repository, or with no arguments to list the tasks available in the current repository.

::: tip
The first time Git-Tool encounters a repository's `git-tool.yml` (and any time it changes), you will
be shown its contents and asked whether you trust it. See [Trust](#trust) below for details.
:::

#### Aliases
 - `gt task`
 - `gt t`
 - `gt run`

#### Example
``` powershell
# Run the 'build' task in the current repository
gt t build

# List the tasks available in the current repository
gt task
```

## Worktree automation <Badge text="v3.11+"/>
The `worktree` section of `git-tool.yml` lets a repository automate the setup of new
[worktrees](dev.md#worktree). When you create a worktree with `gt worktree`, Git-Tool will:

1. Create the requested **symlinks** from the worktree back to the original repository. This is ideal
   for expensive-to-recreate directories like `node_modules` or Rust's `target` which can safely be
   shared between worktrees.
2. Run the listed **tasks** within the context of the new worktree, for example to install
   dependencies or perform an initial build.

```yaml
worktree:
  symlinks:
    - node_modules
    - target
  tasks:
    - install
```

::: tip
On Windows, directory symlinks are created using junctions so that they work without requiring
administrator privileges or developer mode. On Unix-like systems, standard symlinks are used.
:::

::: warning
Worktree automation is only applied once you trust the repository's configuration. If you decline,
Git-Tool will skip the symlinks and tasks but still create and open the worktree as usual.
:::

## trust <Badge text="v3.11+"/>
Running tasks from a `git-tool.yml` file means executing commands defined by that repository, so
Git-Tool maintains a list of repositories whose configuration you trust. This list is stored in your
root [configuration file](../config/README.md) as a map of repository names to the SHA-256 hash of the
`git-tool.yml` contents that you approved.

When you run a task (or create a worktree with automation) for a repository whose configuration has
not been trusted - or whose configuration has changed since you last trusted it - Git-Tool will show
you the configuration and prompt you to decide:

- **always** - trust this exact configuration permanently. The hash is written to your root config so
  you won't be prompted again until the configuration changes.
- **once** - run the tasks this one time without saving your decision.
- **no** - do not run the tasks.

The `gt trust` command lets you manage this list directly.

#### Aliases
 - `gt trust`

#### Options
 - `-r`, `--remove` (alias `--rm`) revokes trust for the specified repository.
 - `-l`, `--list` lists the repositories you currently trust.

#### Example
``` powershell
# Trust the current repository's configuration
gt trust

# Trust a specific repository's configuration
gt trust github.com/sierrasoftworks/git-tool

# Stop trusting a repository
gt trust --remove github.com/sierrasoftworks/git-tool

# List the repositories you currently trust
gt trust --list
```
