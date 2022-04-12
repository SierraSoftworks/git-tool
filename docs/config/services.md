# Services
Git Tool has been written to support a wide range of Git servers and allows you to add your own via the config file.
This enables you to interact with most common hosted Git services as well as your own on-premise ones.

The quickest way to add services to your config is to use the [`gt config add`](../commands/config.md#config-add)
command to install services from the [registry](registry.md). If your service isn't there already, feel free to
add it by following the [contribution guide](registry.md#contributing).

Here is an example service configuration for GitHub which showcases how to 

:::: code-group

::: code-group-item Git-Tool v3.x
```yaml
services:
  - name: gh
    website: "https://github.com/{{ .Repo.FullName }}"
    gitUrl: "git@github.com:{{ .Repo.FullName }}.git"
    pattern: "*/*"
    api:
      kind: GitHub/v3
      url: https://api.github.com
```
:::

::: code-group-item Git-Tool v2.x
```yaml
services:
  - domain: github.com
    website: "https://{{ .Service.Domain }}/{{ .Repo.FullName }}"
    gitUrl: "git@{{ .Service.Domain }}:{{ .Repo.FullName }}.git"
    httpUrl: "https://{{ .Service.Domain }}/{{ .Repo.FullName }}.git"
    pattern: "*/*"
```
:::

::::

## Configuration
::: tip
Git-Tool supports [templates](templates.md) in both the service and app definitions. These can be used to
access information about the repository that is being targeted by a given command.
:::


#### `name`/`domain` <Badge text="required" type="danger"/>
The `name`/`domain` is the unique identifier for this service and will always be the top-level directory
name below which this service's repositories will be stored.

::: tip
Git-Tool v2.x requires you to specify `domain` here, however Git-Tool v3.x uses `name` for the same purpose.
This was done to make a clear distinction between domains and the names you use to refer to your service configurations.
:::

:::: code-group

::: code-group-item Git-Tool v3.x
```yaml
name: github.com
```
:::

::: code-group-item Git-Tool v2.x
```yaml
domain: github.com
```
:::

::::

#### `website` <Badge text="required" type="danger"/>
The `website` property configures the template which is used to generate URLs for a repository's
website view. You should use either `.Repo.FullName` or a combination of `.Repo.Namespace`
and `.Repo.Name` to build the URL.

```yaml
website: "https://github.com/{{ .Repo.Namespace }}/{{ .Repo.Name }}"
```

::: tip
You can use `.Service.Domain` to access the `domain` field for the current service, making
it easier to copy-paste service definitions for similar services running on different domains
(like GitHub Enterprise and Gitea).
:::

#### `gitUrl` <Badge text="required" type="danger" />
The `gitUrl` property is used to generate the Git URL used by git to access this repository.
It will be configured as the `origin` remote on newly created repositories and used to `git clone`
existing repositories. If you need to fix an error with this, you can use the [`gt fix`](../commands/repos.md#fix)
command to help you out.

```yaml
gitUrl: "git@github.com:{{ .Repo.FullName }}.git"
```


#### `httpUrl` <Badge text="required" type="danger"/> <Badge text="removed in v3.x" type="danger" />
The `httpUrl` property is used to generate the HTTPS URL used by git to access this repository.
It will be used if you set the [`http_transport`](features.md#http_transport) feature flag to `true` on Git-Tool v2.x.

```yaml
httpUrl: "https://github.com/{{ .Repo.FullName }}.git"
```

::: warning
This field is no longer supported in Git-Tool v3.x and we've moved to using the `gitUrl` field instead.
If you wish to use HTTPS as your default transport for `git`, make sure that the `gitUrl` follows the
appropriate HTTPS transport format.

A quick way to do this is to use the [`gt config add`](../commands/config.md#config-add) command to add
the relevant HTTPS configuration to your config file, such as `gt config add github-https --name gh --force`.
:::

#### `pattern` <Badge text="required" type="danger"/>
This is a pseudo-glob pattern used by Git-Tool to describe the depth of the directory structure
at which repositories can be found. A pattern of `*/*` implies that repositories can be found at
the second level of directories within the `github.com` root. In some cases, services may have
three (`*/*/*`) or more levels of nesting.

```yaml
pattern: "*/*"
```

#### `api` <Badge text="v3.x"/>
The `api` property is used to tell Git-Tool how to talk to your Git hosting service when creating
new repositories. This is only required if you want to take advantage of Git-Tool's ability to
create a remote repository whenever you run [`gt new`](../commands/repos.md#new).

```yaml
api:
  kind: GitHub/v3
  url: https://api.github.com
```

##### Supported API Kinds

- `GitHub/v3` is used to communicate with a GitHub server supporting the v3 API. Public GitHub uses
  `https://api.github.com` as its endpoint while a private GitHub Enterprise instance uses
  `https://github.example.com/api/v3`.