# Features

In some situations you may want finer grained control over the way Git Tool
behaves. This is accomplished through the Git Tool configuration file and some
of its advanced options.

## `http_transport` <Badge text="v1.2.19+"/> <Badge text="v3.x" type="warning" />

- **Default** `false`

::: warning This feature flag is not supported in Git-Tool `v3.x`, with this
behaviour being controlled by the [`gitUrl`](./services.md#giturl) service
field. :::

By default Git-Tool uses the SSH transport for `git` with URLs like
`git@github.com:sierrasoftworks/git-tool.git`. In some situations, particularly
those where you wish to run without authentication, you may prefer to use git's
HTTPS transport instead.

::: tip Use `gt config feature http_transport true` to turn this flag on
directly from your command line. :::

## `create_remote` <Badge text="v1.0+"/>

- **Default** `true`

Git-Tool will, if this feature is enabled and the `--no-create-remote` option is
not specified, attempt to create a remote repository on your hosting provider
for recognized services when running `gt new`. This can be helpful for users who
don't want to manually set up a GitHub repo - but you can disable it if you
prefer.

::: tip Use `gt config feature create_remote false` to turn this flag off
directly from your command line. :::

## `create_remote_private` <Badge text="v2.0+"/>

- **Default** `true`

When creating a remote repository with `gt n` and the
[`create_remote`](#create-remote) feature enabled, Git-Tool will (by default)
create a _Private_ repo (if your service supports it). You can usually convert a
Private repository to a Public one when you're ready, however if you would
prefer to create Public repos, you can disable this feature flag.

::: tip Use `gt config feature create_remote_private false` to turn this flag
off directly from your command line. :::

## `check_exists` <Badge text="v3.3+"/>

- **Default** `true`

The [`gt new`](../commands/repos.md) command is responsible for creating
repositories which do not exist yet. Unfortunately, sometimes people forget that
they've already created one with the same name and this can lead to unexpected
conflicts. To help avoid this, Git-Tool can check whether a repository already
exists on a supported remote service before attempting to create a new one.

::: tip Use `gt config feature check_exists false` to turn this flag off
directly from your command line. :::

::: warning This feature is not supported for all services. For a service to
support this feature it must include a supported [`api`](./services.md#api)
field in its configuration. :::

## `open_new_repo_in_default_app` <Badge text="v2.1.1+"/>

- **Default** `false`

When this feature flag is enabled, Git-Tool will automatically open newly
created repositories in your default application when running `gt new`. This is
equivalent to passing the `--open` flag.

::: tip Use `gt config feature open_new_repo_in_default_app true` to turn this
flag on directly from your command line. :::

## `always_open_best_match` <Badge text="v3.2+"/>

When this feature flag is enabled, Git-Tool will always open the best matching
repository if there are multiple repositories which may be matched by your
current pattern.

::: Use `gt config feature always_open_best_match true` to turn this flag on
directly from your command line. :::

## `telemetry` <Badge text="v2.1.21+"/>

- **Default** `false`

Git-Tool can send limited telemetry to [Sentry.io](https://sentry.io) and
[Honeycomb](https://honeycomb.io) to try and help us figure out the cause of
crashes and improve the tool for everyone. If you would like to share your
telemetry with us, you can enable this feature flag.

::: tip Use `gt config feature telemetry true` to turn this flag on directly
from your command line. :::

## `check_for_updates` <Badge text="v3.2+"/>

- **Default** `true`

Git-Tool receives regular updates and includes a built in
[`gt update`](../commands/setup.md#update) command which you can use to update
to the latest version. This feature flag controls whether Git-Tool will check
for new updates when you open a repository and let you know about them when you
exit it. This model is designed to avoid any latency penalties, while keeping
you up to date with the latest updates.

## `native_clone` <Badge text="v1.2.18+" /> <Badge text="v2.0+" type="warning"/>

- **Default** `false`

::: warning This feature flag is not supported in Git-Tool `v2.x`, with this
behaviour being the default in all newer versions of Git-Tool. :::

Git-Tool supports using your local `git` executable to clone and initialize
repositories instead of using its built-in git logic. If you wish to use your
local `git` executable, set this feature to `true` in your config. Doing so may
resolve issues with smart-card/YubiKey authentication, or with SSH-Agent on
Windows.
