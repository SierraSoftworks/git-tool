---
description: How to report a problem with Git-Tool to maximize the chances of getting it fixed.
---

# Reporting Errors
Git-Tool is designed to be as robust and safe as possible, however all software
will fail at some point or other. While we try to make understanding failures as
easy as possible, and suggest how you might be able to fix a problem when it does
occur, there will probably be times when you need to raise an issue for help.

We use [GitHub](https://github.com/SierraSoftworks/git-tool/issues/new/choose) to
track issues and will be happy to help with any problems you're facing. That said,
there are a few things you can do to help us do a better job of helping you.

1. Use our [Bug Report](https://github.com/SierraSoftworks/git-tool/issues/new/choose) template on GitHub.
2. Tell us which version of Git-Tool you are using (and try the latest version to see if it solves your problem).
3. If you can reproduce the issue, please run Git-Tool with the `--trace` argument and attach the Trace ID to the issue.
4. Please provide as much context as you can about what you were doing when the error occurred.

::: tip
If you have telemetry reporting enabled, a Trace ID will automatically be generated for you whenever
an error occurs, saving you the hassle of needing to reproduce the problem with `--trace`.

You can enable telemetry by running `gt config feature telemetry true`.
:::