# Config Management

## config <Badge text="v1.5+"/>
The `gt config` command will print your current [`config.yml`](../config/README.md)
file to `stdout`. It's a great way to quickly view your config, save it or share
it.

The `gt config` command also has a series of sub-commands which allow you to
manage your config without needing to kick open a text editor (along with
autocomplete suggestions).

#### Example
```powershell
# Show your current config
gt config
```

### config list <Badge text="v1.5+"/>
Git-Tool has a [registry](../config/registry.md) of useful apps and services
which you can easily add to your configuration. The `gt config list` command
will show you the items available in this registry and you can install
any of them using [`gt config add`](#config-add).

::: tip
Anyone is welcome to contribute their own templates to the Git-Tool registry,
take a look at the [registry](../config/registry.md) documentation for information
on how to do so.
:::

#### Example
```powershell
# List the apps and services which can be added to your config automatically
gt config list
```

### config add <Badge text="v1.5+"/>
If you find something in the Git-Tool [registry](../config/registry.md) which you
want to add to your config, you can use `gt config add` to install it.

#### Options
 - `-f`, `--force` will overwrite any existing apps or services in your config which share
   the same names as those in the template you are installing.

#### Example
```powershell
# Install the Visual Studio developer prompt app
gt config add apps/visualstudio

# Install the GitHub repository service, overwriting it if it exists
gt config add services/github -f
```

### config alias <Badge text="v2.0+"/>
Git-Tool allows you to setup aliases for repositories you use often. These aliases
can give you a short name by which to refer to a repo and prevent confusion about
which one you intended to open if multiple repos match a pattern you provide.

When using any Git-Tool command which expects a repository name, you can provide
the alias instead. For example: `gt o blog`.

::: tip
Aliases are a great way to distinguish between repos with similar or generic names.
Try something like `gt config alias blog github.com/SierraSoftworks/blog`
:::

#### Options
 - `-d`, `--delete` will delete the alias with the provided name from your config.

#### Example
```powershell
# Add an alias for git-tool
gt config alias gt github.com/SierraSoftworks/git-tool

# View the repository name associated with the gt alias
gt config alias gt

# Remove the gt alias
gt config alias -d gt
```

### config feature <Badge text="v2.1.21+"/>
Git-Tool uses [feature flags](../config/features.md) as a means of tweaking behaviour depending on your
individual preferences. This command allows you to quickly view the feature flags
you have set and modify their values, all with lovely autocomplete support.

::: tip
For the full list of feature flags, take a look at the [configuration docs](../config/features.md).
:::

#### Example
```powershell
# Check the status of all of your feature flags
gt config feature

# Disable crash reporting
gt config feature telemetry false

# Check whether crash reporting is enabled
gt config feature telemetry
```

## auth <Badge text="v2.1+"/>
The `gt auth` command allows you to manage the authentication tokens used to connect to remote
repository hosts like GitHub.

::: warning
These access tokens are stored in your local system keychain
for a bit of extra security, however if you are using a shared computer or
are concerned about the physical security of your device, it is best to avoid
this feature.
:::

#### Options
 - `-d`, `--delete` will remove the stored access token for the service you specify.

#### Example
```powershell
# Store an access token for github.com
gt auth github.com

# Store an access token for github.com without using stdin
gt auth github.com --token $GITHUB_TOKEN

# Remove an access token for github.com
gt auth -d github.com
```

## apps <Badge text="v1.0+"/>
The `gt apps` command provides you with a list of all of the applications
you have added to your [configuration](../config/apps.md).

#### Example
```powershell
# List the apps you have added to your configuration
gt apps
```

## services <Badge text="v1.0+"/>
The `gt services` command provides you with a list of all of the services
you have added to your [configuration](../config/services.md).

#### Example
```powershell
# List the services you have added to your configuration
gt services
```