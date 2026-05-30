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

# PTY bytes → Rust-owned terminal (requires ghostty-vt std pool stubs)
RUSTFLAGS='--cfg ghostty_vt_terminal_owned' cargo test -p ghostty-termio --features rust-vt

# Headless SurfaceSession (config + termio + Rust VT)
RUSTFLAGS='--cfg ghostty_vt_terminal_owned' cargo test -p ghostty-core --features rust-vt
# Includes `tests/app_session.rs` and `tests/surface_session.rs` (separate processes).
RUSTFLAGS='--cfg ghostty_vt_terminal_owned' cargo test -p ghostty-ffi --features rust-vt

# Rust tmux viewer behavioral tests (pane capture, session reset, live output)
RUSTFLAGS='--cfg ghostty_vt_terminal_owned' cargo test -p ghostty-vt-tmux-tests
```

## App-owned pilot

```bash
export RUSTFLAGS='--cfg ghostty_vt_terminal_owned'
cargo build -p ghostty-ffi --features rust-vt

zig build -Demit-macos-app=false -Dterminal-rust-owned-app=true -Drustc=$RUSTC
zig build test -Demit-macos-app=false -Dterminal-rust-owned-app=true -Drustc=$RUSTC --summary all
zig build rust-core -Demit-macos-app=false -Dterminal-rust-owned-app=true
```

App-owned mode sets `c_abi` on the Ghostty terminal module so pin/render/wrapper
symbols export for the linked Rust VT object (same bridge as lib-vt, without a
separate `libghostty-vt` artifact). `-Dterminal-rust-owned-app=true` runs the
`rust-core` step and links `target/<triple>/{debug,release}/libghostty_ffi.a`.

## Phase 7 Cargo-primary bootstrap

`src/build/GhosttyRust.zig` exposes `coreStaticLibBuild` and `coreStaticLibPath` so Zig
packaging can depend on a Cargo-built `libghostty_ffi.a` instead of per-module `rustc` objects.

```bash
export RUSTFLAGS='--cfg ghostty_vt_terminal_owned'
cargo build -p ghostty-ffi --features rust-vt

# Opt-in Zig step (does not affect default `zig build`):
zig build -Drust-core-pilot=true -Demit-macos-app=false
```

`coreStaticLibBuild` uses `cargo build` (not per-module `rustc`) as the primary
artifact path. The `rust-core` step runs `cargo build -p ghostty-ffi --features rust-vt` with
`RUSTFLAGS='--cfg ghostty_vt_terminal_owned'`. Cargo-only builds link `ghostty-vt`
`cargo_link_stubs` (render-owned + wrapper symbols); no Zig VT object required for
workspace `cargo build -p ghostty-ffi`. Artifact path: `coreStaticLibPath` in
`GhosttyRust.zig` (`target/<triple>/{debug,release}/libghostty_ffi.a`).

## Phase 8 flip criteria (not yet)

- [x] `coreStaticLibBuild` / `coreStaticLibPath` in `GhosttyRust.zig`
- [x] `zig build -Drust-core-pilot=true` → `rust-core` step (Cargo `ghostty-ffi` staticlib)
- [x] `-Dterminal-rust-owned-app=true` → auto `rust-core` + link `libghostty_ffi.a` in app exe
- [x] Cargo-only `ghostty-ffi` link via `cargo_link_stubs` (no Zig VT object for workspace builds)
- [ ] All `crates/*` tests green on Linux + macOS CI
- [ ] `ghostty-vt` object built via Cargo artifact, not direct `rustc` invoke
- [ ] Zig build reduced to packaging, codegen, and platform shells only
- [ ] Upstream PR slices merged with `PORTING_STATUS.tsv` rows at `rust-ported`

## Push target

Fork: `Frank-III/ghostty` (`git push fork main`). Upstream `ghostty-org/ghostty` requires maintainer access.
