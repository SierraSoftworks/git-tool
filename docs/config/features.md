# Features
In some situations you may want finer grained control over the way Git Tool behaves. This is accomplished through
the Git Tool configuration file and some of its advanced options.

## `http_transport` <Badge text="v1.2.19+"/>
  - **Default** `false`

By default Git-Tool uses the SSH transport for `git` with URLs like `git@github.com:sierrasoftworks/git-tool.git`.
In some situations, particularly those where you wish to run without authentication, you may prefer to use git's HTTPS transport
instead.

::: tip
Use `gt config feature http_transport true` to turn this flag on directly from your command line.
:::

## `create_remote` <Badge text="v1.0+"/>
 - **Default** `true`

Git-Tool will, if this feature is enabled and the `--no-create-remote` option is not specified, attempt to create
a remote repository on your hosting provider for recognized services when running `gt new`. This can be helpful
for users who don't want to manually set up a GitHub repo - but you can disable it if you prefer.

::: tip
Use `gt config feature create_remote false` to turn this flag off directly from your command line.
:::

## `create_remote_private` <Badge text="v2.0+"/>
 - **Default** `true`

When creating a remote repository with `gt n` and the [`create_remote`](#create-remote) feature enabled,
Git-Tool will (by default) create a *Private* repo (if your service supports it). You can usually convert
a Private repository to a Public one when you're ready, however if you would prefer to create Public repos,
you can disable this feature flag.

::: tip
Use `gt config feature create_remote_private false` to turn this flag off directly from your command line.
:::

## `open_new_repo_in_default_app` <Badge text="v2.1.1+"/>
 - **Default** `false`

When this feature flag is enabled, Git-Tool will automatically open newly created repositories in your
default application when running `gt new`. This is equivalent to passing the `--open` flag.

::: tip
Use `gt config feature open_new_repo_in_default_app true` to turn this flag on directly from your command line.
:::

## `telemetry` <Badge text="v2.1.21+"/>
 - **Default** `true`

Git-Tool sends limited telemetry to [Sentry.io](https://sentry.io) when system-level crashes occur
to try and help us figure out the these crashes and improve the tool for everyone. If you would prefer
not to submit this telemetry, you can disable it with this feature flag.

::: tip
Use `gt config feature telemetry true` to turn this flag on directly from your command line.
:::

## `native_clone` <Badge text="v1.2.18+" /> <Badge text="v2.0+" type="warning"/>
 - **Default** `false`

::: warning
This feature flag is not supported in Git-Tool `v2.x`, with this behaviour being the default in all newer versions of Git-Tool.
:::

Git-Tool supports using your local `git` executable to clone and initialize repositories instead of using its built-in git logic.
If you wish to use your local `git` executable, set this feature to `true` in your config. Doing so may resolve
issues with smart-card/YubiKey authentication, or with SSH-Agent on Windows.