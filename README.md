# Git Tool
**Simplify checking out your Git repositories in a structured directory space**

Git Tool is a powerful tool for managing your Git repositories, storing them in
a consistent folder structure and simplifying access when you need it.

## Example

```powershell
# Open the sierrasoftworks/git-tool repo in your default app (bash by default)
# This will clone the repo automatically if you don't have it yet.
git-tool open sierrasoftworks/git-tool

# Open the github.com/sierrasoftworks/git-tool repo in VS Code (if listed in your config)
git-tool open code github.com/sierrasoftworks/git-tool

# Create a new repository and instruct GitHub to create the repo as well, if you
# have permission to do so.
git-tool new github.com/sierrasoftworks/demo-repo

# Show info about the repository in your current directory
git-tool info

# Show information about a specific repository
git-tool info dev.azure.com/sierrasoftworks/opensource/git-tool
```