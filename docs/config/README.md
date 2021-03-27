# Introduction
Git-Tool uses a `yaml` configuration file while allows you to configure everything from where
your repositories and scratchpads are stored, to which applications you use to open them and
which Git hosting services you use to store them.

By default, Git-Tool will attempt to load your configuration from the path provided in your
`GITTOOL_CONFIG` environment variable, however you can override this by passing the `--config`
flag to any Git-Tool command if you wish.

Many config related changes can be conducted using Git-Tool's [`gt config` command](../commands/config.md),
including adding new services and apps, configuring your feature flags and aliasing your commonly
used repos.

## Directory
The first thing to set up in your `config.yml` file is your development directory path. This is
the directory into which Git-Tool will place your repositories whenever you clone or create them.

:::: code-group
::: code-group-item Windows
```yaml
directory: "C:\\Users\\bpannell\\dev"
```
:::

::: code-group-item Linux
```yaml
directory: "/home/bpannell/dev"
```
:::

::: code-group-item MacOS
```yaml
directory: "/Users/bpannell/dev"
```
:::
::::


## Scratchpads
If you plan on using Git-Tool's [scratchpads](../commands/scratch.md) feature, you might decide that
you want to place your scratchpads in a different directory to your repositories. Maybe you'd like
them to be replicated using your cloud storage service, or maybe you like to live dangerously and
want them on a `/tmp` RAMDisk.


::: tip
If you don't specify a `scratchpads` directory, Git-Tool will use a `scratch` folder within your development
directory to hold your scratchpads.
:::

:::: code-group
::: code-group-item Windows
```yaml
scratchpads: "C:\\Users\\bpannell\\scratch"
```
:::

::: code-group-item Linux
```yaml
scratchpads: "/home/bpannell/scratch"
```
:::

::: code-group-item MacOS
```yaml
scratchpads: "/Users/bpannell/scratch"
```
:::
::::


## Example Configuration
Here is a short example configuration file which you can use as the basis for your own.
You might find the [config commands](../commands/config.md) useful to make changes to it.

::: tip
To view your current configuration, run `gt config`.
:::

```yaml
---
# yaml-language-server: $schema=https://schemas.sierrasoftworks.com/git-tool/v1/config.schema.json
$schema: https://schemas.sierrasoftworks.com/git-tool/v1/config.schema.json
directory: /home/bpannell/dev
services:
  - domain: github.com
    website: "https://{{ .Service.Domain }}/{{ .Repo.FullName }}"
    httpUrl: "https://{{ .Service.Domain }}/{{ .Repo.FullName }}.git"
    gitUrl: "git@{{ .Service.Domain }}:{{ .Repo.FullName }}.git"
    pattern: "*/*"
  - domain: dev.azure.com
    website: "https://{{ .Service.Domain }}/{{ .Repo.Namespace }}/_git/{{ .Repo.Name }}"
    httpUrl: "https://{{ .Service.Domain }}/{{ .Repo.Namespace }}/_git/{{ .Repo.Name }}"
    gitUrl: "git@ssh.{{ .Service.Domain }}:v3/{{ .Repo.FullName }}.git"
    pattern: "*/*/*"
apps:
  - name: shell
    command: pwsh
  - name: code
    command: code
    args:
      - .
aliases:
  gt: github.com/SierraSoftworks/git-tool

features:
  http_transport: false
  open_new_repo_in_default_app: true

  # Disable this if you don't want to report crashes to us
  telemetry: true
```
