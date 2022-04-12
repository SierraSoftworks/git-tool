# Git Tool

**Simplify checking out your Git repositories in a structured directory space**

Git Tool is a powerful tool for managing your Git repositories, storing them in
a consistent folder structure and simplifying access when you need it. The best
place to get started is by reading our [documentation](https://git-tool.sierrasoftworks.com).

## Features

- **Quickly open repositories** whether they are already cloned locally or not, using your favourite Git services and a concise folder structure.
- **Launch applications** within the context of your repositories quickly and consistently.
- **Weekly scratchpads** to help organize random work and doodles with minimal effort.
- **Aliases** to make opening your most common repositories as quick as possible.
- **Fast autocompletion** on all platforms with support for "sequence search" (`ssgt` matches `SierraSoftworks/git-tool`) as found in Sublime and VSCode.

## Example

```bash
# Open the sierrasoftworks/git-tool repo in your default app (bash by default)
# This will clone the repo automatically if you don't have it yet.
gt o sierrasoftworks/git-tool

# Open the github.com/sierrasoftworks/git-tool repo in VS Code (if listed in your config)
gt o code gh:sierrasoftworks/git-tool

# Create a new repository and instruct GitHub to create the repo as well, if you
# have permission to do so.
gt new gh:sierrasoftworks/demo-repo

# Show info about the repository in your current directory
gt i

# Show information about a specific repository
gt i ado:sierrasoftworks/opensource/git-tool

# Open your shell in the current week's scratch directory
gt s
```
