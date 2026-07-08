---
name: verify
description: Build and drive the git-tool CLI to verify a change end-to-end, including observing telemetry events locally.
---

# Verifying git-tool changes

## Run the test suite

```bash
# Use pure-tests to avoid network calls and dependencies on the keychain, which can be flaky in sandboxed shells.
cargo test --features pure-tests
```

## Build & run

```bash
cargo build                        # binary at target/debug/git-tool
GITTOOL_CONFIG=<config.yml> ./target/debug/git-tool <command>
```

Minimal test config (put dev/scratchpads under a temp dir):

```yaml
directory: /tmp/gt-verify/dev
scratchpads: /tmp/gt-verify/scratchpads
features:
  telemetry: true          # off by default; required for event delivery
  check_for_updates: false # avoids GitHub API calls on `gt open`
apps:
  - name: shell
    command: "true"        # overrides default shell; exits instantly
```

Useful flows: `gt list` (resolve many), `gt scratch` (new-folder task + app
launch/exit), `gt update --state '<garbage>'` (error path, exits 1).

## Observing telemetry events

The analytics endpoint is hard-coded in `src/telemetry/mod.rs`. Temporarily
point `tracing_batteries::Analytics::new(...)` at `http://127.0.0.1:8377`,
run a tiny HTTP listener that dumps POST bodies (`/track/hit`,
`/track/exception`), drive commands, and read the captured JSON. **Revert the
URL afterwards.** Events appear as `"e":"custom"` hits with the event name in
`"n"` and properties in `"d"`.

## Gotchas

- `cargo test` fails on every test that creates a git commit when the global
  git config enables SSH commit signing and no signing agent is reachable
  (sandboxed shells): commits hang ~60s then exit 128. Fix by overriding for
  the run:

  ```bash
  GIT_CONFIG_GLOBAL=<neutral-gitconfig> GIT_CONFIG_SYSTEM=/dev/null cargo test --features pure-tests
  ```

  where the neutral config sets `user.name`/`user.email`,
  `commit.gpgsign = false` and `init.defaultBranch = main`.

