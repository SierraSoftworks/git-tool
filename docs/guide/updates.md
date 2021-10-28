# Updating Git-Tool
Git Tool includes support for self-updating, however it doesn't do this automatically. Here's how to perform an update of your Git Tool installation quickly and easily from your command line.

:warning: **NOTE** This process will only work when upgrading from (and to) versions of Git Tool >= `1.4.0`.

## Listing Updates
To view the list of updates, you can run:

```
λ gt update --list
 * v1.4.0
 - v1.3.2
 - v1.3.1
 - v1.3.0
 - v1.2.30
 - v1.2.29
 - v1.2.28
 - v1.2.27
```

Your current version will be indicated with a `*`.

## Updating
When you decide to update, you can do so using `gt update` or `gt update VERSION` depending on your needs.

If there are no newer updates available, you will be presented with the following message:
```
λ gt update
No update available
```

If there is a new update available, you will be shown the following:

```
λ gt update
Downloading update v1.4.0...
Shutting down to allow update to proceed
```

At this stage, if there are no other running instances of Git Tool on your system, the update will take place.
If you have other running instances, you will need to exit them before the update can complete. **We will never automatically shut down other Git Tool instances as they may be running important tasks.**

If you wish, you can also update to a specific application version by doing the following:

```
λ gt update v1.4.1
Downloading update v1.4.1...
Shutting down to allow update to proceed
```

This allows you to install beta versions of Git Tool to test out new features before everyone else (as Git Tool will never automatically update a stable release to a beta release).

## Why isn't this automatic?
Good question, we've considered making updates automatic however there are a few reasons why we've decided not to.

 - Git Tool is a power-user tool and we believe you should be in control of when an update gets applied.
   - This also allows you to skip troublesome updates or rollback to a previous version should you encounter problems.
 - Git Tool is designed to be extremely quick, delaying an operation even for the second that it takes to get a list of available updates flies in the face of that.