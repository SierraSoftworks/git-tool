# Introduction
Git-Tool was originally designed as a means of organizing your filesystem and removing the need
to think about where a given repository should be stored. The goal was to let you hop right into
working on a project without breaking your mental context around why you wanted to work on it
in the first place.

::: tip
For a quick introduction to using Git-Tool's core commands, take a look at the
[Usage Guide](../guide/usage.md).
:::

Over the years, Git-Tool has expanded in scope, but its core goal remains the same: organize
your development work automatically so you don't need to think about it and can focus on the
important stuff.

## Repositories
Git-Tool automatically manages the location of your local repositories. When you create a new
repo, it will `git init` the appropriate folder, configure your `git remote`s and (if your hosting
service is supported) create a new repo there too. When you want to open a repo, Git-Tool will
automatically `git clone` it (if it doesn't already exist) - so you never need to worry about
whether a repo is present locally or not.

[Read more about managing your repos &rarr;](repos.md)

## Scratchpads
Scratchpads are weekly directories intended for all those little things which need a place on
your filesystem but probably won't matter a few weeks from now. I use them for everything from
quick experiments, to trying out new tools and sometimes even for taking notes.

[Read more about using scratchpads &rarr;](scratch.md)

## Development
When using git, there are a few things we all find ourselves doing a lot. Things like maintaining
our `.gitignore` files or switching between branches (which might not always exist locally).
To make your life a bit easier, Git-Tool includes support for [gitignore.io](https://gitignore.io)
and a `git switch` command proxy with great auto-complete.

[Read more about to use Git-Tool in your repos &rarr;](dev.md)

## Config Management
Git-Tool has a wealth of configuration options available and discovering them all can be
daunting. To make that a bit easier on everyone, Git-Tool provides a CLI to make common
config changes, saving us all a bit of time and grey hair.

[Read more about tweaking your config &rarr;](config.md)
