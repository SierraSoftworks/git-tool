---
description: Some frequently asked questions which you might find useful.
---

# Frequently Asked Questions

## Default Git branch names

When you create a new repository with Git-Tool, it will be initialized using
`git init`. This will, by default, create a branch called `master` but with
you might wish to use something else. For a while, Git-Tool allowed you to use `main` instead, however with Git now allowing you to configure the default branch name, you can now use anything you wish.

To set this up, you should run the following command.

```bash
git config --global init.defaultBranch main
```