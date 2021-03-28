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