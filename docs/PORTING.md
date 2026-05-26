# Ghostty Zig to Rust Porting Guide

This guide is for the incremental Ghostty port. The goal is a port, not a
rewrite: keep Ghostty's architecture, data structures, public C ABI, build
behavior, and tests recognizable while replacing Zig implementation slices with
Rust only where the replacement can be validated.

The Bun Rust port is a useful process reference, especially its phase discipline:
draft from the existing implementation, make compile boundaries explicit, fix
real root causes, and run adversarial validation. Do not copy Bun's crate layout
or runtime choices into Ghostty by default.

## Current Scope

Start with `libghostty-vt` and terminal C API slices. This boundary is small
enough to validate with `zig build test-lib-vt`, and it already has public C
headers and examples that make ABI regressions visible.

The current Rust integration is opt-in:

- Zig option: `-Dlib-vt-rust=true`
- Rust object source: `src/terminal/rust/lib.rs`
- Zig build integration: `src/build/GhosttyRust.zig`
- C ABI wrappers: `src/terminal/c/*.zig`

Rust code is compiled as a raw object and linked into the existing Zig build.
Do not introduce a Cargo workspace until a concrete slice needs it and the
build contract is written down first.

## Ground Rules

- The sibling Zig implementation is the spec. Read the Zig before writing Rust.
- Preserve the public C ABI. Do not change headers in `include/ghostty/vt/`
  unless the requested slice explicitly changes the public API.
- Keep allocation ownership where it already lives. For early C API slices,
  keep `new` and `free` in Zig unless the slice is specifically about
  allocation.
- `unsafe` is allowed. Use it to preserve layout, pointer, and ABI semantics,
  not to skip understanding the Zig invariant.
- Do not replace C or vendored library behavior with Rust crates. If Zig calls
  through an existing C/C++ dependency, Rust should usually call that same
  dependency.
- Prefer narrow, completed slices over broad scaffolding. A slice is not done
  until both the Rust-backed and default Zig-backed paths pass targeted tests.
- Never hide incomplete code behind broad stubs, fake success values, or
  disabled build gates. If something is incomplete, leave the Zig path active.

## Slice Shape

For a terminal C API slice:

1. Identify the exported C wrapper in `src/terminal/c/`.
2. Read the Zig type and any lower-level Zig module it wraps.
3. Add layout tests before Rust reads or writes Zig-owned memory.
4. Add `extern fn ghostty_rust_*` declarations behind
   `terminal_options.lib_vt_rust`.
5. Keep Zig enum validation and error translation unless the Rust slice
   explicitly ports that logic.
6. Implement the Rust export in `src/terminal/rust/lib.rs`.
7. Run targeted tests with and without `-Dlib-vt-rust=true`.

For layout-sensitive fields, assert the exact facts Rust depends on:

- `@offsetOf` for every field touched through a raw pointer.
- `@sizeOf` for every by-value ABI type.
- Optional and packed representations when Rust writes them directly.
- Enum integer width and sentinel rules for public C headers.

Non-`extern` Zig structs may reorder fields. Do not infer layout from source
field order.

## FFI And Layout

Rust exports should use explicit C-compatible types:

- `core::ffi::{c_int, c_void}` for C values.
- `#[repr(C)]` for structs passed by value through C ABI.
- `#[repr(transparent)]` only when the Zig/C ABI is truly one-field
  transparent.
- Raw pointers for Zig-owned memory.

Be careful with layout mismatches:

- Zig `?*T` is pointer-sized; Rust `Option<*mut T>` is not the same guarantee.
- Zig optional enums can have a separate tag and payload. Measure before
  writing.
- Small by-value structs can expose ABI edge cases. If Debug Zig probes show an
  ABI hazard, keep that function in Zig and choose another slice.
- If Rust touches unaligned storage, use the appropriate unaligned pointer
  operation or keep access in Zig.

Do not create Rust references from raw pointers unless the aliasing guarantee is
real for the whole reference lifetime. A raw pointer plus per-access read/write
is often the faithful port of Zig pointer semantics.

## Build Rules

The pinned Zig version is `0.15.2` in `build.zig.zon`; treat that as part of
the port contract.

Use repo build steps, not ad hoc `zig test` commands:

- Targeted lib-vt test:
  `zig build test-lib-vt -Dtest-filter=<filter>`
- Targeted Rust-backed lib-vt test:
  `zig build test-lib-vt -Dtest-filter=<filter> -Dlib-vt-rust=true`
- Full Rust-backed lib-vt test:
  `zig build test-lib-vt -Dlib-vt-rust=true --summary all`
- lib-vt archive build:
  `zig build -Demit-lib-vt -Dlib-vt-rust=true -Demit-macos-app=false --summary all`

For Rust raw-object checks, compile `src/terminal/rust/lib.rs` with the same
environment variables supplied by `GhosttyRust.zig`, then run `nm -u` on the
object. Undefined symbols must be intentional and explained.

## Validation Contract

Every completed slice should record:

- Zig source path read as the spec.
- Rust exports added.
- Layout assertions added or reused.
- Targeted command without Rust flag.
- Targeted command with `-Dlib-vt-rust=true`.
- Broader lib-vt command when the slice touches shared behavior.
- Any ABI or layout hazard found and how it was handled.

Passing tests are evidence only for the behavior they cover. If a Rust slice
depends on a representation detail, the representation assertion is part of the
test surface.

## Commit Discipline

Use commit messages that make review possible from history:

- `port(vt): route mouse_event accessors through Rust`
- `port(vt): add Rust build_info backing`
- `port(vt): assert key_event wrapper layout`

Mention root cause and validation in the body for bug-fix commits. Avoid
commits that mix porting, cleanup, formatting, and unrelated refactors.

## Reference Notes From Bun

Bun's useful process patterns for Ghostty:

- Start with a written porting guide before widening the port.
- Keep original implementation files as the semantic reference during the port.
- Make crate/build graph boundaries explicit before moving large areas.
- Treat dependency cycles as architecture signals, not hook opportunities.
- Group test failures by crash signature and fix the root cause.
- Review fixes against the original Zig semantics.
- Remove internal migration notes once the repo no longer needs them.

Ghostty should apply those patterns at Ghostty scale: narrow terminal/lib-vt
slices first, GUI/app layers later.

### Bun PR 30412 Audit

Reference checkout:

- Local path: `references/guided/bun-pr-30412`
- Remote ref: `refs/pull/30412/head`
- Local branch: `pr-30412`
- Inspected material: commit subjects, commit bodies, Markdown/TSV docs, and
  workflow prompts only. Do not use this checkout as a source-code template.

The PR history is intentionally phase-shaped:

- `phase-a`: draft Rust files from Zig files in deterministic batches.
- `phase-b0`: analyze crate dependency cycles, then move code or introduce
  explicit dispatch boundaries.
- `phase-b1`: make the Rust workspace compile crate-by-crate with gates and
  stubs only as temporary scaffolding.
- `phase-b2`: replace gates with real bodies, committing verification fixes as
  part of the port.
- `phase-c` and later: link/smoke, harvest systemic issues, then run swarms for
  TODOs, unsafe audits, test crashes, Windows parity, and main-branch parity.

The strongest lesson for Ghostty is the review loop, not the scale:

- Write down port rules before widening the surface.
- Classify pointer ownership from evidence, not from type spelling alone.
- Treat dependency cycles as misplaced ownership or dispatch boundaries.
- Keep verification adversarial: compare Rust behavior to the Zig spec and
  reject suppressions, fake success, or broad re-gating.
- Record root causes in commit subjects/bodies so history is searchable.

Bun's internal `docs/PORTING.md` evolved several hard bans that transfer well to
Ghostty:

- Do not reimplement existing C/C++ or vendored-library behavior in Rust.
- Do not use `Box::leak`, `mem::forget`, or `ManuallyDrop` to force lifetimes.
- Do not use `todo!()`, `unimplemented!()`, or `#[cfg(any())]` as porting
  stubs.
- Do not solve layering problems with `extern "Rust"` hooks or runtime
  function-pointer registration when moving code or threading data is the real
  fix.
- Prefer raw pointers over invented Rust references when the original Zig
  contract allows aliasing.
