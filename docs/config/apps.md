# Applications

Git Tool has the ability to launch applications within the context of your repositories. This is useful when you want to
quickly open a shell or your favourite editor and start working, however you can also add a wide range of other applications
there.

The quickest way to add applications to your config is to use the [`gt config add`](../commands/config.md#config-add)
command to install apps from the [registry](registry.md). If your favourite app isn't there already, feel free to
add it by following the [contribution guide](registry.md#contributing).

::: tip
The first application in your config file will be launched by default if you do not specify an application
in your command.
:::

Here's an example of an app which uses all of the configuration options to launch a Visual Studio
developer command prompt in your repo. For more information on what each of these properties do, keep reading.
```yaml
apps:
  - name: vs
    command: powershell.exe
    args:
      - "-NoExit"
      - "-Command"
      - "& { Import-Module VSSetup; $vs = Get-VSSetupInstance | Select-VSSetupInstance -Latest; Import-Module (Join-Path $vs.InstallationPath $env:VSDEVSHELL_PATH); Enter-VsDevShell -VsInstallPath $vs.InstallationPath -StartInPath '{{ .Target.Path }}' }"
    environment:
      - VSDEVSHELL_PATH=Common7\\Tools\\Microsoft.VisualStudio.DevShell.dll
```


## Configuration
Applications in your Git-Tool configuration file have a `name` and `command`. The `name` is what you
will use when launching the application (such as in `gt o my-app my-repo`) and the `command` is the
program which will be executed when you do.

#### `name` <Badge text="required" type="danger"/>
This is the name of the app which you will use when running commands like `gt o my-app` and `gt s my-app`.
Usually it's a good idea to make this short and easy to type.

```yaml
name: bash
```

#### `command` <Badge text="required" type="danger"/>
This is the program which will be launched by Git-Tool for this app entry. You can use the name of the program
if it exists on your `$PATH` or an absolute path if it does not.

```yaml
command: /usr/bin/bash
```

::: tip
Git-Tool will set the current working directory of the program to that of the scratchpad or repo you are
launching it in. If you need to change this, you can do so using a spawned shell and `cd`.
:::

#### `args`
If you wish to pass arguments to your program, you can provide an array of them here. This can be very useful
for setting up a shell environment (if you need that) or simply for avoiding the creation of a wrapper script.

```yaml
args:
 - '-c'
 - 'echo $MESSAGE'
```

#### `environment`
Sometimes you need to set up environment variables for the program you are launching and this
is the way to do it. Simply provide an array of environment variables you wish to expose to
the application (overriding those in Git-Tool's environment if they conflict) and you're good to go.

```yaml
environment:
 - 'MESSAGE=Hello {{ .Target.Name }}'
```

::: tip
**Did you notice that `.Target.Name`?** Git-Tool's apps (and services) support [templates](templates.md),
so you can gather information about the repository or scratchpad you're targeting and tailor your call appropriately.
:::

