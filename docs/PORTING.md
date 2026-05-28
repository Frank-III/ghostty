# Ghostty Zig to Rust Porting Guide

This guide is for the incremental Ghostty port. The goal is a port, not a
rewrite: keep Ghostty's architecture, data structures, public C ABI, build
behavior, and tests recognizable while replacing Zig implementation slices with
Rust only where the replacement can be validated.

The Bun Rust port is a useful process reference, especially its phase discipline:
draft from the existing implementation, make compile boundaries explicit, fix
real root causes, and run adversarial validation. Do not copy Bun's crate layout
or runtime choices into Ghostty by default.

## Current Port Status

**VT milestone:** `libghostty-vt` Rust port is default-ready on validated macOS/iOS
targets. C ABI helpers are Rust-backed unless `-Dlib-vt-rust=false`.

**Whole-project Rust rewrite:** Early. A Cargo workspace exists under `crates/`
(Phase 0 bootstrap). The Ghostty application, renderer, font stack, config, and
`libghostty` embedding API remain Zig-owned. macOS Swift and GTK apprt shells
stay native; the target is a Rust core library with stable C embedding APIs.

### Project inventory

| Phase | Subsystem | Zig anchor | Rust crate | Status |
|------:|-----------|------------|------------|--------|
| 0 | VT / libghostty-vt | `src/terminal/rust/` | `crates/ghostty-vt` | **Done** (C ABI default-on) |
| 0 | Rust terminal ownership | `src/terminal/c/terminal.zig` | `terminal_owned.rs` | **Opt-in** (`-Dterminal-rust-owned=true`) |
| 1 | Foundation | `src/datastruct/`, `unicode/`, `simd/`, `lib/`, `os/` | `ghostty-foundation` | In progress |
| 2 | Config | `src/config/` | `ghostty-config` | In progress (minimal loader) |
| 3 | Input | `src/input/` | `ghostty-input` | Not started |
| 3 | Termio / PTY | `src/termio/`, `pty.zig`, `Command.zig` | `ghostty-termio` | Not started |
| 4 | Font | `src/font/` | `ghostty-font` | In progress (metrics, descriptor, discovery skeleton) |
| 5 | Renderer | `src/renderer/` | `ghostty-renderer` | Not started |
| 6 | App / Surface / embed | `App.zig`, `Surface.zig`, `apprt/embedded.zig` | `ghostty-core`, `ghostty-ffi` | Not started |
| 7 | CLI / inspector / crash | `src/cli/`, `inspector/`, `crash/` | `ghostty-cli`, `ghostty` | Not started |
| 8 | Build system | `build.zig`, `src/build/` | Cargo primary | Hybrid (Zig orchestrates, Cargo workspace bootstrapped) |
| — | macOS UI | `macos/` (Swift) | — | **Keep** (links Rust `libghostty` when ready) |
| — | Linux UI | `src/apprt/gtk/` | — | **Keep** (thin glue over Rust core) |

### Cargo workspace

```bash
# Check stub crates and ghostty-vt sources
cargo check --workspace

# Zig still builds the shipping artifacts
zig build -Demit-lib-vt -Demit-macos-app=false
```

The `ghostty-vt` crate reuses `src/terminal/rust/lib.rs` as its library root.
Zig continues to invoke `rustc` directly via `src/build/GhosttyRust.zig` for
shipping builds; Cargo is the long-term primary build (Phase 8).

### Rust terminal ownership (opt-in)

Pool/page bootstrap FFI lives in `src/terminal/c/pin_bridge.zig`. Rust-owned
terminals compile only with `-Dterminal-rust-owned=true` (sets
`ghostty_vt_terminal_owned` cfg). Exports:

- `ghostty_rust_terminal_create` / `destroy` / `write`

The main Ghostty app enables `-Dlib-vt-rust` for the terminal module and links
the Rust VT object (`src/build/SharedDeps.zig`).

### Validation (VT)

```bash
# Build default Rust-backed lib-vt. Use an explicit rustc to avoid PATH surprises.
zig build -Demit-lib-vt -Demit-macos-app=false \
  -Drustc=$HOME/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc \
  --summary all

# Run lib-vt tests with Rust backing by default.
zig build test-lib-vt \
  -Drustc=$HOME/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc \
  --summary all
```

Latest verified result with the explicit rustc above:

- Build summary: `80/80 steps succeeded`.
- Test summary: `22/22 steps succeeded`; `4233/4251` tests passed, `18`
  skipped.
- Rust-backed test leg: `2277` passed, `9` skipped.
- Rust compiler warnings: `0`.

On local machines, plain `zig build ...` uses `rustc` from `PATH`; pass
`-Drustc=...` when the ambient toolchain is not reliable.

### Modules Ported (~50+ Rust files in `src/terminal/rust/`)

**Foundation types**: `ansi.zig` → `ansi.rs`, `size.zig` → `size_types.rs`,
`point.zig` → `point.rs`, `style.zig` → `style_types.rs`, `cursor.zig` →
`cursor_style.rs`

**Parsing infrastructure**: `UTF8Decoder.zig` → `utf8_decoder.rs`,
`Parser.zig` + `parse_table.zig` → `vt_parser.rs`, `charsets.zig` →
`charsets.zig`, `csi.zig` → `csi.zig`, `modes.zig` → `mode_def.rs`

**Stream layer**: `stream.zig` split across → `stream_types.rs`,
`stream_handler.rs`, `stream_core.rs`, `stream_csi_dispatch.rs`,
`stream_esc_dispatch.rs`, `stream_osc_dispatch.rs`, `stream_osc_parse.rs`,
`sgr_attribute.rs`, `osc_types.rs`

**Data structures**: `bitmap_allocator.zig` → `bitmap_allocator.rs`,
`hash_map.zig` → `hash_map.rs`, `ref_counted_set.zig` → `ref_counted_set.rs`,
`Tabstops.zig` → `tabstops.rs`

**Page buffer** (the core cell/row storage):
`page.zig` → `page_types.rs` (Cell/Row bitpacked u64 types),
`page_core.rs` (Page struct + layout + init),
`page_methods.rs` (cell move/swap/clear, hyperlink/grapheme/style operations,
clone)

**Page list + linked list**: `PageList.zig` → `page_list_types.rs` +
`page_list_methods.rs` (2225 lines: linked list, Pin movement, iterators,
resize with reflow via `ReflowCursor` from `reflow_cursor.rs`)

**Screen + Terminal state**: `Screen.zig` → `screen_types.rs` +
`screen_methods.rs` (2165 lines: cursor mgmt, erase, scroll, resize,
SGR attributes). `Terminal.zig` → `terminal_types.rs` + `terminal_methods.rs`
(1055 lines: modes, write loop, full reset). `stream_terminal.zig` →
`stream_terminal.rs` (94 StreamHandler methods dispatching to Terminal)

**Formatter**: `formatter.zig` → `formatter_types.rs`, `formatter_terminal.rs`,
`formatter_screen.rs`, `formatter_page.rs`

**Kitty graphics protocol**: `kitty/*.zig` → `kitty_graphics_command.rs`,
`kitty_graphics_exec.rs`, `kitty_graphics_image.rs`, `kitty_graphics_storage.rs`,
`kitty_graphics_unicode.rs`

**Other**: `apc.zig`, `dcs.zig`, `hyperlink.zig`, `highlight.zig`,
`Selection.zig`, `ScreenSet.zig`, `StringMap.zig`, `x11_color.zig`,
`tmux/`, `search/`, `kitty/color.zig`, OSC parser modules, device attributes,
device status, mouse shape, SGR attribute types

### Known Gaps

**PNG decoding** (`kitty_graphics_image.rs:decode_png`): Supported when the
host installs a decoder with
`ghostty_sys_set(GHOSTTY_SYS_OPT_DECODE_PNG, ...)`. The Rust path calls through
the system bridge and returns `ImageError::UnsupportedFormat` only when no
decoder is installed.

**Tracked pins** (`page_list_methods.rs`): Functional through the Zig
MemoryPool FFI bridge. Rust allocates/destroys pins through
`ghostty_vt_pin_create` / `ghostty_vt_pin_destroy` and stores the tracked-pin
array in Zig-owned pool memory.

**MemoryPool fields** (page list types): `PageListMemoryPool.alloc`,
`.nodes`, `.pages`, and `.pins` remain `*mut c_void` — they are Zig-owned
handles that Rust forwards back to Zig and never dereferences directly.

**Kitty graphics file loading** (`kitty_graphics_image.rs`): A POSIX
implementation reads files via `open`/`fstat`/`read` and shared memory via
`shm_open`/`mmap`. The validated target set is currently macOS/iOS. Linux
validation is still required before removing the Unix platform caveat. Windows
paths return `ImageError::UnsupportedMedium`.

**tmux integration** (`tmux/`): `tmux_control.rs`, `tmux_layout.rs`,
`tmux_output.rs`, and `tmux_viewer.rs` are ported. `tmux_viewer.rs` now routes
pane output, pane capture, and pane state into Rust `Terminal` instances where
the viewer owns those handles. This area still needs behavioral coverage beyond
the generic `libghostty-vt` suite.

### Intentionally Unsupported

These behaviors return explicit error values and are not port gaps:

- **File-based kitty graphics on Windows**. Returns
  `ImageError::UnsupportedMedium`.
- **Shared-memory kitty graphics on Windows**. Returns
  `ImageError::UnsupportedMedium`.
- **Animation frame playback**. Returns an `unimplemented` error from the
  Kitty graphics execution path.
- **Highlight lifecycle cleanup** outside the C ABI terminal-core path.
  This remains deferred until the broader app/surface highlight ownership
  boundary moves with the port.

### Build Contract

- `#![no_std]` crate with `panic = "abort"` — no standard library, no unwinding.
  `core::` only; no heap allocator is pulled in from `std`.
- `GhosttyAllocator` (in `allocator.rs`) is the only allocator: it wraps Zig's
  vtable allocator through FFI (`GhosttyAllocatorVtable` with `alloc`/`resize`/
  `free` function pointers). All Rust-land collections flow through it so
  Zig-owned memory stays on Zig's ledger.
- Debug `opt-level = 1` required. At `opt-level = 0` the compiler emits a
  reference to the `panic_bounds_check` lang item, which no_std panic=abort
  builds don't provide and which the linker rejects. `GhosttyRust.zig` sets
  this explicitly for Debug builds.
- No `#[derive(Debug)]`. `core::fmt` formatter symbols referenced by `Debug`
  impls are not reachable from the final link in some configurations.
- No `assert!`/`unwrap()`/`expect()`/`[]` indexing on the same linker grounds
  when they would pull in panic-fmt symbols; use explicit bounds-checked
  accessors and `if`/`match` branches that don't panic.
- All unsafe operations in explicit `unsafe {}` blocks
  (`#![deny(unsafe_op_in_unsafe_fn)]`).
- All types crossing the FFI boundary use `#[repr(C)]`. `extern "C"` for
  Zig-linked functions.
- Zig sibling `.zig` files kept in `src/terminal/rust/` as semantic reference
  (not compiled — Bun PR 30412 pattern).
- System libc is linked (mmap, shm_open, fstat, etc. work). System libz is
  linked for zlib decompression.




Start with `libghostty-vt` and terminal C API slices. This boundary is small
enough to validate with `zig build test-lib-vt`, and it already has public C
headers and examples that make ABI regressions visible.

The current Rust integration is default-on for libghostty-vt C ABI builds:

- Zig option: `-Dlib-vt-rust=true` / `-Dlib-vt-rust=false`
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
