# Fuzzing

Run the template fuzzer for 60 seconds:

```console
./scripts/fuzz
```

Choose another target and duration:

```console
./scripts/fuzz commands 300
```

Arguments after the duration are passed to libFuzzer. For example, this runs
the checked-in template corpus once:

```console
./scripts/fuzz templates 0 -runs=1
```

The script requires the nightly Rust toolchain and `cargo-fuzz`:

```console
rustup toolchain install nightly
cargo install cargo-fuzz
```

Generated corpus entries are written to `fuzz/corpus-output/<target>`. The
checked-in files under `fuzz/corpus/<target>` are used as read-only seeds.

Set `FUZZ_SANITIZER` to select a cargo-fuzz sanitizer explicitly. Sanitizers
default to disabled on Apple Silicon because the instrumented binary currently
fails to link there; other platforms use cargo-fuzz's default sanitizer.