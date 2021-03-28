# Registry
Writing your own [service](services.md) and [application](apps.md) entries can sometimes
be a bit more work than we're feeling like and since Git-Tool is meant to make your life
easier, not harder, we figured it would be a good idea to try and simplify this part for
you as well.

To solve the problem, we added a central registry of config templates which you can search
through and install with the [`gt config add`](../commands/config.md#config-add) command.
This makes the process of setting up your Git-Tool config as easy as doing the following:

```powershell
# Get a list of all the apps and services available to me
gt config list

# Add my favourite apps
gt config add apps/powershell
gt config add apps/powershell-admin
gt config add apps/vscode
gt config add apps/visualstudio

# Add the services I use
gt config add services/github
gt config add services/azure-devops
```

## Browse

To get the latest list of apps and services in the registry, you can always use
[`gt config list`](../commands/config.md#config-list) straight from your command
line. Of course, here's the list if you're interested :wink:.

<ClientOnly>
    <RegistryBrowser />
</ClientOnly>

## Contributing
Thanks for choosing to contribute to the Git-Tool community :heart:! We'd like to make this as
easy as possible, so keep reading for 
We're so happy that you're considering contributing an app or service config to Git-Tool's registry.
In theory, all you need to do is write a `yaml` file and submit a PR to our [GitHub repo][git-tool]
to get it added to the `registry` folder. We have some automated tests in place which should help
ensure that what you are submitting is valid and those run locally as part of the standard Git-Tool
test suite (run with `cargo test` if you have Rust installed).

### Registry
Git-Tool's registry is simply a folder hosted on [GitHub][git-tool]. Specifically, it's the `registry`
folder in the Git-Tool repo. Within this folder are folders for `apps` and `services` to help keep
things organized. 

For those who prefer a visual representation of what the registry looks like.

<FileTree>

 - [github.com/SierraSoftworks/git-tool][git-tool]
   - registry/
     - apps/
       - bash.yaml
       - *your-app.yaml* &larr; :rocket:
     - services/
       - github.yaml
       - *your-service.yaml* &larr; :rocket:
</FileTree>

### Example Template
Here is an example of what a registry template might look like and you are welcome to use it as
the basis for your own. Keep reading for more information on what each field does and how to
use them (or just wing it, if you're already familiar with how Git-Tool's [apps](apps.md) and
[services](services.md) are defined).

::: warning
We usually avoid bundling apps and services into a single file, but if you've got
a compelling reason to do so - then we can certainly make an exception. *The example below
includes both apps and services to show how to use them, not because it's a good idea.*
:::

```yaml
# yaml-language-server: $schema=https://schemas.sierrasoftworks.com/git-tool/v1/template.schema.json
name: Demo
description: This is an example of how to create a config template
version: 1.0.0
configs:
  # Your config should include either a service (like this)...
  - platform: any
    service:
      domain: github.com
      website: "https://{{ .Service.Domain }}/{{ .Repo.FullName }}"
      httpUrl: "https://{{ .Service.Domain }}/{{ .Repo.FullName }}.git"
      gitUrl: "git@{{ .Service.Domain }}:{{ .Repo.FullName }}.git"
      pattern: "*/*"

  # Or an app (like this) but usually not both.
  - platform: windows
    app:
      name: shell
      command: powershell.exe

  # You can also add platform specific versions of each app
  - platform: linux
    app:
      name: shell
      command: pwsh
```

### Template Structure
Registry templates are `yaml` files (with the `.yaml` extension) which Git-Tool will use to
update your local config. They have a bit of metadata to explain to humans what they do, but
the most important part is the list of `configs` which tell Git-Tool how to modify your
local [config](README.md) file.

::: tip
We publish a [JSONSchema](https://json-schema.org) schema for Git-Tool templates which your
editor can use to give you autocomplete and automatic validation. To include it, just add the
following to the top of your template.

```yaml
# yaml-language-server: $schema=https://schemas.sierrasoftworks.com/git-tool/v1/template.schema.json
```
:::

#### `name` <Badge type="danger" text="required"/>
This is the human readable name you wish to give to this template. It doesn't need to match
the names given to [apps](apps.md) or [services](services.md) contained within, but it usually
should be pretty close.

```yaml
name: PowerShell Core
```

#### `description` <Badge type="danger" text="required"/>
The description is used to explain to humans what your template will add and why that might
be of use to them. If possible, use [plain English](https://en.wikipedia.org/wiki/Plain_English)
and assume that the reader will not be familiar with the tool or service you are adding.

```yaml
description: |
    Launches the PowerShell Core shell as your current user.
    PowerShell Core must be installed on your platform before use.
    
    https://github.com/PowerShell/PowerShell/releases
```

#### `version` <Badge type="danger" text="required"/>
The version is used to show humans when you have updated this template and it should
follow [SemVer](https://semver.org) conventions. Currently Git-Tool doesn't keep track of this
field, but in future we may add support for updating the items you have installed from
the registry using this field.

```yaml
version: 1.0.0
```

#### `configs` <Badge type="danger" text="required"/>
This is where the heart of the template fits in. The `configs` field is a list (array) of
config templates which Git-Tool will apply to your [config](README.md) file. These templates
can either be for an [app](apps.md) or a [service](services.md) and require that you specify
the `platform` that they support. Keep reading for details on what fields you use within
each config.

```yaml
configs:
  - platform: any
    app:
      name: shell
      command: pwsh
```

##### `configs.*.platform` <Badge type="danger" text="required"/>
When describing a config template, you need to provide the platform that it supports.
This allows Git-Tool to apply the right template in situations where different platforms
require different configuration.

The list of supported platform types includes:

 - `any` will apply to any platform.
 - `windows` will only apply to Windows platforms.
 - `linux` will only apply to Linux platforms.
 - `darwin` will only apply to Mac OS X platforms.

```yaml
configs:
  - platform: linux
```

##### `configs.*.app` <Badge type="warning" text="optional"/>
When creating a config template which adds an [app](apps.md), you will use the
`app` field to provide an application definition as you would in your normal
[config file](apps.md). All of the normal [app](apps.md) fields are supported.

::: warning
If you specify the `app` field, you will not be able to provide the `service`
field in the same entry. Add a new item to the `configs` array if you need to do this.
:::

```yaml
configs:
  - app:
      name: shell
      command: bash
      args: [] # Optional
      environment: [] # Optional
```

##### `configs.*.service` <Badge type="warning" text="optional"/>
When creating a config template which adds a [service](services.md), you will use the
`service` field to provide a service definition as you would in your normal
[config file](services.md). All of the normal [service](services.md) fields are supported.


::: warning
If you specify the `app` field, you will not be able to provide the `service`
field in the same entry. Add a new item to the `configs` array if you need to do this.
:::

```yaml
configs:
  - service:
      domain: github.com
      website: "https://{{ .Service.Domain }}/{{ .Repo.FullName }}"
      httpUrl: "https://{{ .Service.Domain }}/{{ .Repo.FullName }}.git"
      gitUrl: "git@{{ .Service.Domain }}:{{ .Repo.FullName }}.git"
      pattern: "*/*"
```

### Creating a PR
The easiest way to get started adding a new [app](apps.md) or [service](services.md)
to the registry is by using the GitHub web editor to create your template and submit
a PR.

 - [Add a new service &rarr;](https://github.com/SierraSoftworks/git-tool/new/main/registry/services)
 - [Add a new app &rarr;](https://github.com/SierraSoftworks/git-tool/new/main/registry/apps)

Fill in the name of your app or service (this is the name people will use to install it, so keep it short but
descriptive) and add your template. Once you're done, create a new PR for your change and we'll
get to reviewing it for you!

#### Automated Testing
Our automated test suite on GitHub will check your PR to make sure that your template can be
loaded by Git-Tool correctly and will warn you if there are any problems.

You can also run this same test suite locally if you have `rust` installed on your
machine by cloning the Git-Tool repo and running `cargo test` in it. If you already have
Git-Tool set up, this is as easy as:

```powershell
# Replace this with your Git-Tool fork, if you have one
gt o github.com/SierraSoftworks/git-tool
cargo test
```

[git-tool]: https://github.com/SierraSoftworks/git-tool


<script>
import FileTree from "../../../components/FileTree.vue"
import RegistryBrowser from "../../../components/Registry.vue"

export default {
  components: {
    FileTree,
    RegistryBrowser
  }
}
</script>