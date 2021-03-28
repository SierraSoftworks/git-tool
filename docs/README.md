---
home: true

actions:
    - text: Get Started
      link: /guide/
    - text: Download
      link: https://github.com/SierraSoftworks/git-tool/releases
      type: secondary

features:
    - title: Organized
      details: |
        Stop trying to figure out which folder to store your project in. You already know where it'll be on GitHub,
        Git-Tool will take care of the rest.

    - title: Seamless Clones
      details: |
        Git-Tool will make sure your repository is ready to go when you need it, no more messing around with Git URLs.

    - title: Scratchpads
      details: |
        Just because your doodles are disorganized, doesn't mean your filesystem needs to be. Git-Tool gives you
        weekly directories for your doodles in just 5 keystrokes!
---


Git Tool is a developer productivity toolset designed to enable command line developers to more quickly manage
and interact with their various repositories. It is written in Go and supports command completion in most common
shells.

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


<ClientOnly>
    <Contributors repo="SierraSoftworks/git-tool" />
</ClientOnly>

<script>
import Contributors from "../../components/Contributors.vue"

export default {
  components: {
    Contributors
  }
}
</script>