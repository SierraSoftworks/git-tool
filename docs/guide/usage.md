# Basic Usage
Git-Tool has a ton of useful commands, but you can probably get away with
knowing only three of them. Let's run through `gt open`, `gt new` and `gt scratch`
and show you how they work!

## Opening a Repo
So, you've just got Git-Tool installed and want to kick the tyres. The best place
to start is the same place you usually do, cloning a repo and opening it up.

```powershell
gt o github.com/SierraSoftworks/git-tool
```

This will automatically clone the repo if it doesn't already exist on your
machine and then open the first application in your [config](../config/apps.md)
file.

You can also launch a specific application if you'd prefer.

```powershell
gt o code github.com/SierraSoftworks/git-tool
```

::: tip
Take a look at Git-Tool's config [registry](../config/registry.md) for a quick way
to add applications to your config.
:::

## Creating a new Repo
Okay, opening a repo is pretty easy - but how about creating a new one? Well,
Git-Tool takes care of all of that for you too. Let's try creating a quick test
repo to play around in.

```powershell
gt n --no-create-remote github.com/YOURUSER/git-tool-example1
```

If you'd like to open up the repo you just created (usually we do) you can either
pass the `-o` command line option, or you can set the
[`open_new_repo_in_default_app`](../config/features.md#open-new-repo-in-default-app)
feature flag.

```powershell
# Tell Git-Tool to automatically open all new repos in your default app
gt config feature open_new_repo_in_default_app true

# Or pass the -o option explicitly
gt n -o --no-create-remote github.com/YOURUSER/git-tool-example2
```

::: tip
We're using the [`--no-create-remote`](../commands/repos.md#new) option here to prevent Git-Tool from automatically
setting up a GitHub repo, but you can leave that out when creating real repositories.
You can also permanently disable this feature by disabling the 
[`create_remote`](../config/features.md#create-remote) feature flag.
:::

## Toying around
We just saw a great example of wanting to play around with something but not
really wanting to create repositories for it. That tends to happen quite a lot
and Git-Tool has just the command to help make it easier. Meet [Scratchpads](../commands/scratch.md),
the organized, weekly, folder where you can do exactly that.

To open the current week's scratchpad, just run the following.

```powershell
# Open the current week's scratchpad
gt s

# Open a different week's scratchpad
gt s 2021w10
```

As with the `gt open` command, [`gt scratch`](../commands/scratch.md) lets you open
the scratchpad in a specific application if you would prefer.

```powershell
gt s shell
```

## More
Of course, this is just the tip of the iceberg, take a look at the
[commands reference](../commands/README.md) to get an idea of everything Git-Tool can do.