# Fuzzing

Fuzzing uses [`cargo-afl`](https://rust-fuzz.github.io/book/afl/tutorial.html)
(AFL++). Run the template fuzzer for 60 seconds:

```console
./scripts/fuzz
```

Choose another target and duration:

```console
./scripts/fuzz commands 300
```

Arguments after the duration are passed to `afl-fuzz`. For example, this limits
each execution to a 10 second timeout:

```console
./scripts/fuzz templates 60 -t 10000
```

The script requires a stable Rust toolchain and `cargo-afl`:

```console
cargo install cargo-afl
cargo afl config --build
```

`cargo afl config --build` installs the AFL++ tooling that `cargo afl build`
relies on the first time you fuzz.

The checked-in files under `fuzz/corpus/<target>` are used as read-only seed
inputs. AFL writes its session state and any discovered crashes to
`fuzz/corpus-output/<target>` (which is git-ignored). Crashes are stored under
`fuzz/corpus-output/<target>/default/crashes`.

Reproduce a crash by piping it back into the target binary:

```console
cargo afl run --release --manifest-path fuzz/Cargo.toml --bin templates \
    < fuzz/corpus-output/templates/default/crashes/<crash-file>
```

The script sets `AFL_SKIP_CPUFREQ`, `AFL_NO_AFFINITY` and
`AFL_I_DONT_CARE_ABOUT_MISSING_CRASHES` so it runs non-interactively in CI and
on machines where the kernel core pattern or CPU governor cannot be
reconfigured. It also sets `AFL_BENCH_UNTIL_CRASH` so a run stops as soon as a
reproducible crash is found. Override any of these by exporting them before
invoking the script.