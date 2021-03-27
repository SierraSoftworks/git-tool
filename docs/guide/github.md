# Linking to GitHub
Git-Tool <3 GitHub and has some first class integrations to make working with it a
little bit easier. One of these features is the ability to automatically create GitHub
repositories when you run `gt new`.

## Installing the Service
If you don't already have the `github.com` service in your [config](../config/services.md)
then you'll want to go and install it by running:

```powershell
gt config add services/github
```

## Authenticating
To enable the automatic creation of GitHub repositories, you'll need to be authenticated. This is
done by creating a [new Personal Access Token](https://github.com/settings/tokens/new?scopes=repo)
and providing that to Git-Tool.

The Personal Access Token you generate needs to have the following scopes:

 - `repo`

Once you've generated the new Personal Access Token, run the [`gt auth`](../commands/config.md#auth)
command and paste the token there.

```powershell
gt auth github.com
```

::: warning
Git-Tool stores your access tokens in your system keychain to help keep them safe from prying eyes,
but this doesn't stop someone with physical access to your computer from finding them, so please
be careful and only use this on trusted devices.
:::

## Configuration
If you would prefer not to create GitHub repositories by default, you can disable the
[`create_remote`](../config/features.md#create-remote) feature by running
`gt config feature create_remote false`.

By default, we'll create **Private** repositories on GitHub, however you can create
Public repositories instead by disabling the [`create_remote_private`](../config/features.md#create-remote-private)
feature by running `gt config feature create_remote_private false`.