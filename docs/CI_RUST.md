# Rust port CI checklist (Phase 8 bootstrap)

This documents the validation bundle to run before pushing Rust port work.
Full Cargo-primary CI is not flipped yet; Zig still builds shipping artifacts.

## Required on every VT change

```bash
export RUSTC=$HOME/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc

zig build test-lib-vt -Dterminal-rust-owned=true -Drustc=$RUSTC --summary all
zig build test-lib-vt -Dterminal-rust-owned=false -Drustc=$RUSTC --summary all
zig build -Demit-lib-vt -Demit-macos-app=false -Drustc=$RUSTC --summary all
```

Expected: **4234/4252** tests passed, **18** skipped, **0** failed, **0** leaked.

## Workspace crates

```bash
RUSTDOC=$HOME/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustdoc \
  cargo test --workspace --exclude ghostty-vt
```

## App-owned pilot (opt-in)

```bash
zig build -Demit-macos-app=false -Dterminal-rust-owned-app=true -Drustc=$RUSTC
```

## Phase 8 flip criteria (not yet)

- [ ] All `crates/*` tests green on Linux + macOS CI
- [ ] `ghostty-vt` object built via Cargo artifact, not direct `rustc` invoke
- [ ] Zig build reduced to packaging, codegen, and platform shells only
- [ ] Upstream PR slices merged with `PORTING_STATUS.tsv` rows at `rust-ported`

## Push target

Fork: `Frank-III/ghostty` (`git push fork main`). Upstream `ghostty-org/ghostty` requires maintainer access.
