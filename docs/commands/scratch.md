# Scratchpads
Scratchpads are Git-Tools version of dumping all of your "unsorted" work onto your
desktop because you can't be bothered to figure out a better place to put them. Well,
almost the same, except that it keeps the chaos neatly organized by week so you can
impress co-workers with your mad organization skills. Don't worry, we won't tell them
if you don't.

#### Directory Structure
The naming scheme used to generate scratchpads is `YEAR-WEEK`, so you'll end up with
directories like `2021w10` for the 10th week of 2021. This works well when it comes
to maintaining context for your current week, with the idea that you probably don't
care too much about things left in your scratchpads from 6 months ago. Of course,
if you decide you want to promote something to a proper project, you can always
use [`gt new`](repos.md#new) to give it its own repository.

<FileTree>

 - scratch
   - 2020w48
   - 2021w10
   - 2021w11
</FileTree>


## scratch <Badge text="v1.2.8+"/>
The `gt scratch` command opens a weekly scratchpad for you to work in. It's a great
place for you to toss things you're hacking on, notes you're taking or just to have
somewhere relatively organized to play around with a new toy.

It works very similarly to the [`gt open`](repos.md#open) command, it just doesn't use git.
That means you can still launch any of your configured applications just as you would
if you were dealing with a repository. Have fun :smile:.

#### Aliases
 - `gt scratch`
 - `gt s`


#### Example
```powershell
# Open the current week's scratchpad in your default app
gt s

# Open a specific week's scratchpad in your default app
gt s 2021w10

# Open the current week's scratchpad in VS Code
gt s code

# Open a specially named scratchpad folder
gt s 2021w10-super-important
```

::: tip
You don't need to use our naming scheme if you don't want to, just run `gt s something` and
we'll create a `something` folder for you with no complaints. *This can be useful if you
have an important project which you don't want to lose track of.*
:::

## temp <Badge text="v3.6.0+" />
The `gt temp` command is a shortcut for opening a temporary scratchpad which is automatically
deleted when you close the launched application. This is useful when you want to quickly test
something without leaving a mess behind.

::: tip
The `gt temp` command creates a temporary folder in your current user's temporary directory
and launches the specified application in that folder. When the application exits, the folder
will be automatically deleted.
:::

#### Example
```powershell
# Open a temporary scratchpad in your default app
gt t

# Open a temporary scratchpad in PowerShell
gt temp pwsh

# Open a temporary scratchpad and don't delete it when you're done
gt temp --keep
```

::: warning
Applications which exit immediately after launching (like VSCode's `code` command) will cause
the temporary folder to be deleted immediately after the application is launched. You can prevent
this by using the `--keep` flag to prevent the folder from being deleted - however you will need
to manually clean it up when you're done (in this case it works much the same as a standard scratchpad).
:::
