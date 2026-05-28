//! Ghostty binary entry (Phase 7/8). Replaces `src/main_ghostty.zig` over time.

fn main() {
    // Hybrid build: the production binary is still produced by `zig build` until Phase 8.
    eprintln!("ghostty Rust binary stub — use `zig build` for the full application");
    std::process::exit(1);
}
