---
description: How to migrate from Git-Tool v2.x to v3.x quickly and easily.
---

# Migrating to v3.x

Git-Tool v3.x is a major release which makes some fundamental changes to the way
that we configure services for Git-Tool and how we choose where to place them on
the filesystem. This guide will walk you through the process of updating your
existing Git-Tool configuration to work with Git-Tool v3.x.

This document shows an example of an updated configuration, and then describes
the specific changes you should expect to make when updating to v3.x.

For more information on why we've made these changes, take a look at the
[why we're changing things](#why-are-we-changing-things) section.

## Example

Here is an example configuration file which has been updated to reflect the
changes introduced in Git-Tool v3.x.

```diff
---
directory: C:\\dev
services:
-  - domain: github.com
-    website: "https://{{ .Service.Domain }}/{{ .Repo.FullName }}"
-    gitUrl: "git@{{ .Service.Domain }}:{{ .Repo.FullName }}.git"
-    httpUrl: "https://{{ .Service.Domain }}/{{ .Repo.FullName }}.git"
+  - name: gh
+    website: "https://github.com/{{ .Repo.FullName }}"
+    gitUrl: "git@github.com:{{ .Repo.FullName }}.git"
     pattern: "*/*"
+    api:
+      kind: GitHub/v3
+      url: https://api.github.com

-  - domain: dev.azure.com
-    website: "https://{{ .Service.Domain }}/{{ .Repo.Namespace }}/_git/{{ .Repo.Name }}"
-    gitUrl: "git@{{ .Service.Domain }}:v3/{{ .Repo.FullName }}.git"
-    httpUrl: "https://{{ .Service.Domain }}/{{ .Repo.Namespace }}/_git/{{ .Repo.Name }}.git"
+  - name: ado
+    website: "https://dev.azure.com/{{ .Repo.Namespace }}/_git/{{ .Repo.Name }}"
+    gitUrl: "git@ssh.dev.azure.com:v3/{{ .Repo.FullName }}.git"
     pattern: "*/*/*"
apps:
  - name: shell
    command: sh
    default: true
  - name: code
    command: code.cmd
    args:
      - .
  - name: make
    command: make
    args:
      - build
    environment:
      - CI_SERVER=0
      - REPO={{ .Repo.FullName }}
aliases:
-  gt: github.com/SierraSoftworks/git-tool
+  gt: gh:SierraSoftworks/git-tool
```

## Changes

### Configuration Schema

Git-Tool's configuration schema has been updated from v1 to v2. There are
several key changes, including the following:

1. The `domain` field has been renamed to `name`.
2. The `httpUrl` field has been removed in favour of using the `gitUrl` field to
   determine whether HTTP transport is used.
3. The `api` field has been added to enable the creation of remote repositories
   on GitHub Enterprise servers.
4. The removal of the `http_transport` feature flag, which is no longer
   supported.

```yaml{3-10}
# yaml-language-server: $schema=https://schemas.sierrasoftworks.com/git-tool/v2/config.schema.json
services:
    # The new layout for a service entry in your configuration file
  - name: gh
    website: "https://github.com/{{ .Repo.FullName }}"
    gitUrl: "git@github.com:{{ .Repo.FullName }}.git"
    pattern: "*/*"
    api:
      kind: GitHub/v3
      url: https://api.github.com

    # The original layout for a service entry in your configuration file
  - domain: github.com
    website: "https://{{ .Service.Domain }}/{{ .Repo.FullName }}"
    gitUrl: "git@{{ .Service.Domain }}:{{ .Repo.FullName }}.git"
    httpUrl: "https://{{ .Service.Domain }}/{{ .Repo.FullName }}.git"
    pattern: "*/*"
```

### Specifying Services

Git-Tool v3.x changes the way you specify a service as part of a repository's
name.

```diff
# v2.x
- github.com/sierrasoftworks/git-tool
# v3.x
+ gh:sierrasoftworks/git-tool
```

This affects the way you use commands like
[`gt open`](../commands/repos.md#open) and [`gt new`](../commands/repos.md#new)
and will require that you update your alias definitions to match.

```diff
  aliases:
-   gt: github.com/sierrasoftworks/git-tool
+   gt: gh:sierrasoftworks/git-tool
```

### Folder Structure

As a side-effect of changing the naming scheme used by services in Git-Tool, you
may need to move your repositories around on disk. This is only required if you
wish to switch to the new naming scheme (`github.com` &rarr; `gh`).

To do so, simply rename the corresponding service directories inside your
development directory to match the new service `name` field.

## Why are we changing things?

Git-Tool, as with all software, is a constant work in progress and as we
continue to use it we find new and fun ways to break it, or edge cases that we
didn't consider when we first designed it. Over time, these have become the
paper-cuts that have pushed people away from using it as their one-stop solution
for repository management.

We're hoping that by making these changes, we alleviate many of these pain
points and make Git-Tool a more useful tool for everyone. Some of the key issues
that kept cropping up included:

1. **Anonymous Pulls from GitHub**

   GitHub's public repositories play really nicely with anonymous HTTPS cloning,
   however they tend to get uppity when you use an SSH key if you aren't one of
   the maintainers. A quick fix for this was to switch on the `http_transport`
   feature flag temporarily, but we often don't want to rely on that for our
   private repositories.

   Being in a position to easily choose which transport we use for a given repo
   would massively simplify this, but using `domain` as an identifier for a
   service and separating the `gitUrl` and `httpUrl` fields made doing so quite
   the hack.

   To make this a bit cleaner, we opted to rename and simplify these fields to
   make that workaround the primary, supported, way of running things.

2. **Supporting GitHub Enterprise**

   One of Git-Tool's coolest features is the way that
   [`gt new`](../commands/repos.md#new) integrates with GitHub, automatically
   creating a new repository whenever you run the command. Unfortunately,
   coupling this functionality to the domain of the service in your config
   really wasn't the most flexible way to handle things and meant that even
   though GitHub Enterprise uses the same API, we didn't include support for it.

   By separating the notion of "which API should I use" from the concept of
   which domain we connect to, we can make supporting GitHub Enterprise much
   easier and unlock the door for similar integrations on other self-hosted Git
   platforms.

3. **Long File Paths**

   Git-Tool's default folder structure is great when it comes to being able to
   navigate and find what you're looking for, but it is rather verbose. This was
   a particularly large problem for Windows users who didn't have Long Path
   Support enabled (limiting them to 256 characters).

   By shortening the ID used to identify a service, we can save a few extra
   characters and hopefully avoid making their lives too hard.

4. **Name Resolution Conflicts**

   With us allowing arbitrary names for services, it was likely that a user
   would add a service called `bob` and then try to add a repository called
   `bob/repo` to their GitHub service. We'd now struggle to determine whether
   they were asking for the `gh:bob/repo` or the `bob:repo` repository. To
   prevent this, we've changed the separator between service and repository to
   be `:` instead of the original `/`, making it clear which component we're
   referring to.
