# Development
Git-Tool is capable of more than just file organization, its great
cross-platform autocomplete support makes it ideally suited to solving
some other common problems we run into during our day-to-day development.

## switch <Badge text="v2.1.16+"/>
Git has recently shipped with a wonderful new [`switch`](https://git-scm.com/docs/git-switch)
command which simplifies changing branches (when compared to the sometimes
confusing `git branch` and `git checkout -B` combination). We're big fans of
Git-Tool's autocomplete though, so we've wrapped all the lovely git goodness
up in Git-Tool and paired it with our search engine to make your life just that
much easier.

::: warning
This command has replaced the old `gt branch` command since `v2.2.0` as it serves the same
purpose and uses the same command line arguments.
:::

#### Aliases
 - `gt switch`
 - `gt sw`
 - `gt branch`
 - `gt b`
 - `gt br`

#### Options
 - `-N`, `--no-create` prevents the branch from being created if it doesn't exist already.

#### Example
``` powershell
# Checks out the feature/demo branch, creating it if it doesn't exist yet
gt sw feature/demo

# Checks out the feature/demo branch if it exists
gt b -N feature/demo
```

## worktree <Badge text="v3.10+"/>
If you find yourself wanting to work on several branches of a repository at the same time,
git's [worktree](https://git-scm.com/docs/git-worktree) feature is invaluable. The `gt worktree`
command brings the same ergonomics you get from `gt open` and `gt switch` to worktrees: from within
a repository it creates a worktree for the branch you've asked for and then launches your chosen
application inside it.

Worktrees are stored within the [`worktrees` directory](../config/README.md#worktrees) configured in
your config file, which defaults to a `worktrees` folder inside your development directory. Each
worktree is placed in its own folder named after the repository and branch, with a short hash of the
repository's full name appended to keep repositories with the same short name from colliding.

::: tip
Run `gt worktree` from within a repository to operate on the current repo (`gt w <branch> [app]`),
just like `gt switch`.

If you don't specify a branch, Git-Tool will list the existing worktrees for the repository. The
primary working tree is labelled `[primary]`, and worktrees with a detached `HEAD` are shown with
their current commit.
:::

::: tip Worktree automation
A repository can automate the setup of new worktrees - creating symlinks (for directories like
`node_modules` or `target`) and running setup tasks - by adding a `worktree` section to its
`git-tool.yml` file. See [Worktree automation](tasks.md#worktree-automation) for details.
:::

::: warning Windows long paths
Worktree directories are nested inside your `worktrees` folder and include the repository name,
branch name and a hash suffix, so the resulting paths can become quite long. On Windows you may
run into the default 260 character path limit (`MAX_PATH`). If you do, either choose a shorter
`worktrees` directory (for example `C:\wt`) or
[enable long path support](https://learn.microsoft.com/windows/win32/fileio/maximum-file-path-limitation#enable-long-paths-in-windows-10-version-1607-and-later)
in Windows and git (`git config --global core.longpaths true`).
:::

::: warning
Using `--rm` will only remove the worktree if all of your changes have been committed. If you have
uncommitted changes, Git-Tool will leave the worktree in place and let you know so that you don't
lose any work — commit or discard your changes and then remove it with `git worktree remove <path>`.

Branches created for throwaway worktrees are left behind on purpose; use [`gt prune`](#prune)
to clean them up once they've been merged.
:::

#### Aliases
 - `gt worktree`
 - `gt w`
 - `gt wt`

#### Options
 - `-N`, `--no-create` prevents the branch from being created if it doesn't exist already.
 - `--base` controls the branch that a newly created worktree branch is based on.
 - `--rm` removes the worktree once the launched application exits.

#### Example
``` powershell
# Create (or open) a worktree for the current repository's feat/forgejo branch
gt w feat/forgejo

# Open the feat/forgejo worktree in VS Code
gt w feat/forgejo code

# Base a new worktree branch on origin/main
gt w feat/forgejo --base origin/main

# Open a throwaway worktree that is removed once you exit the shell
gt w feat/forgejo --rm

# List the existing worktrees for the current repository
gt w
```

## ignore <Badge text="v1.0+"/>
Setting up your `.gitignore` files and keeping them updated can be a bit
of a faff. It takes time, it doesn't add much core value and we often forget
little things when then require us to clean out repo history.

[gitignore.io](https://gitignore.io) is one approach to solving this. It
offers a wide range of pre-built `.gitignore` files for different languages
and frameworks. Git-Tool gives you a command which combines this with
some useful metadata to allow quick updates and additions.

#### Aliases
 - `gt ignore`
 - `gt gitignore`

#### Options
 - `--path` allows you to specify the path to the `.gitignore` file you wish to update. By default, this will be `./.gitignore`.

#### Example
``` powershell
# View the list of all supported .gitignore languages/frameworks
gt ignore

# Add a .gitignore file (or update your existing one) for Node.js and C#
gt ignore node csharp
```

::: tip
Git-Tool will automatically fetch the latest ignore files from [gitignore.io](https://gitignore.io)
for the languages you have added whenever you run `gt ignore $LANG` and retain changes you made outside
the metadata blocks.
:::


## prune <Badge text="v2.3.0+"/>
If you're the kind of person who uses branches and Pull Requests to make changes
to your repositories, you'll often end up with a small horde of branches in your
local repository which are remnants of previously merged PRs. Cleaning these out
can be a bit of a chore, so Git-Tool provides a `prune` command which will identify
any merged branches (using `git branch --merged`) and remove them for you automatically.

In addition to merged branches, `prune` will remove any Git worktrees for the
repository which do not contain uncommitted changes. Worktrees with pending work
are left in place so that you don't lose anything, and you'll be asked to confirm
before anything is removed.

You can optionally provide one or more patterns to only prune branches and
worktrees whose branch name contains one of those patterns, and use the
`--branches`/`--worktrees` flags to restrict the operation to just branches or
just worktrees (by default both are pruned).

#### Options
 - `-y`, `--yes` will skip the confirmation prompt and remove any branches that are found.
 - `-b`, `--branches` will only prune merged branches (worktrees are left untouched).
 - `-w`, `--worktrees` will only prune clean worktrees (branches are left untouched).
 - `[pattern]...` restricts pruning to branches and worktrees whose branch name contains one of the provided patterns.

#### Example
``` powershell
# Remove any merged branches from your local repository
gt prune

# Only remove merged branches whose name contains "feature/"
gt prune --branches feature/

# Only remove clean worktrees
gt prune --worktrees
```