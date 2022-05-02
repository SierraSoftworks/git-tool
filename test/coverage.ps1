$env:RUSTFLAGS = "-C instrument_coverage"
$env:LLVM_PROFILE_FILE = "default.profraw"
cargo test

Write-Host "Merging raw profile output files"
&"$(rustc --print target-libdir)/../bin/llvm-profdata" merge -sparse default.profraw -o default.profdata

$latest_asset = Get-ChildItem -Path ./target/debug/deps -Filter "git_tool-*" -File `
| Where-Object { $_.Name.EndsWith(".exe") -or (-not $_.Name.Contains(".")) } `
| Sort-Object -Top 1 -Property LastWriteTime

Write-Host "Latest Asset: $latest_asset"

Write-Host "Exporting LCOV coverage report"
&"$(rustc --print target-libdir)/../bin/llvm-cov" export -instr-profile default.profdata $latest_asset `
    -Xdemangler=rustfilt `
    -ignore-filename-regex='.cargo|rustc' `
    -compilation-dir=src `
    -format=lcov > lcov.info