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
Pop it into your `$PATH` and you're good to go!

::: tip
At this point, you'll be able to run `git-tool` to open repositories, create new ones,
or manage your weekly scratchpads without any further configuration, however you'll
probably want to continue through the remaining steps to get the most out of your setup.
:::

#### Step #2: [Setup](../commands/setup.md)
To get the most out of Git-Tool, you should run the setup wizard with `git-tool setup`.
This wizard will guide you through the process of setting up your configuration file
and shell autocompletion to ensure you get the most out of Git-Tool.

#### Step #3: [Linking to GitHub](github.md)
Git-Tool <3 GitHub and can automatically create repositories there whenever you run `gt new`.

To set this up, generate a [new Personal Access Token](https://github.com/settings/tokens/new?scopes=repo)
with the `repo` scope and run the following command to store it in your local keychain.

```powershell
gt auth gh
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