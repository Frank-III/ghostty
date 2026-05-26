const std = @import("std");
const Config = @import("Config.zig");
const TerminalBuildOptions = @import("../terminal/build_options.zig").Options;

pub fn libVtObject(
    b: *std.Build,
    cfg: *const Config,
    options: TerminalBuildOptions,
    name: []const u8,
) std.Build.LazyPath {
    const target = cfg.target.result;
    const triple = rustTargetTriple(target) orelse
        @panic("no Rust target mapping for this libghostty-vt target; pass -Dlib-vt-rust=false");

    const run = b.addSystemCommand(&.{
        cfg.rustc,
        "--crate-name",
        name,
        "--crate-type=lib",
        "--edition=2021",
        "--emit=obj",
        "--target",
        triple,
        "-C",
        "panic=abort",
        "-C",
        "relocation-model=pic",
        "-C",
        "debug-assertions=no",
    });
    addBuildInfoEnv(run, b, cfg, options);

    switch (cfg.optimize) {
        .Debug => run.addArgs(&.{ "-C", "opt-level=0", "-C", "debuginfo=2" }),
        .ReleaseSafe => run.addArgs(&.{ "-C", "opt-level=3", "-C", "debuginfo=1" }),
        .ReleaseFast => run.addArgs(&.{ "-C", "opt-level=3" }),
        .ReleaseSmall => run.addArgs(&.{ "-C", "opt-level=z" }),
    }

    run.addArg("-o");
    const output = run.addOutputFileArg(objectName(target));
    run.addFileArg(b.path("src/terminal/rust/lib.rs"));
    for (lib_vt_modules) |module| {
        run.addFileInput(b.path(module));
    }
    return output;
}

const lib_vt_modules = [_][]const u8{
    "src/terminal/rust/allocator.rs",
    "src/terminal/rust/build_info.rs",
    "src/terminal/rust/cell.rs",
    "src/terminal/rust/color.rs",
    "src/terminal/rust/constants.rs",
    "src/terminal/rust/early.rs",
    "src/terminal/rust/event_key.rs",
    "src/terminal/rust/event_key_action.rs",
    "src/terminal/rust/event_key_codepoint.rs",
    "src/terminal/rust/event_key_field.rs",
    "src/terminal/rust/event_key_mods.rs",
    "src/terminal/rust/event_key_utf8.rs",
    "src/terminal/rust/event_mouse.rs",
    "src/terminal/rust/event_mouse_action.rs",
    "src/terminal/rust/event_mouse_button.rs",
    "src/terminal/rust/event_mouse_field.rs",
    "src/terminal/rust/event_mouse_position.rs",
    "src/terminal/rust/focus.rs",
    "src/terminal/rust/grid_ref.rs",
    "src/terminal/rust/input.rs",
    "src/terminal/rust/key_encode.rs",
    "src/terminal/rust/kitty_geometry.rs",
    "src/terminal/rust/kitty_graphics.rs",
    "src/terminal/rust/kitty_graphics_geom.rs",
    "src/terminal/rust/kitty_image.rs",
    "src/terminal/rust/kitty_placement_get.rs",
    "src/terminal/rust/kitty_placement.rs",
    "src/terminal/rust/lib.rs",
    "src/terminal/rust/modes.rs",
    "src/terminal/rust/mouse_button.rs",
    "src/terminal/rust/mouse_encoder_state.rs",
    "src/terminal/rust/mouse_geometry.rs",
    "src/terminal/rust/mouse_encode.rs",
    "src/terminal/rust/mouse_types.rs",
    "src/terminal/rust/mouse_setopt.rs",
    "src/terminal/rust/mouse_setopt_bool.rs",
    "src/terminal/rust/mouse_setopt_mode.rs",
    "src/terminal/rust/mouse_setopt_size.rs",
    "src/terminal/rust/mouse_write.rs",
    "src/terminal/rust/osc.rs",
    "src/terminal/rust/paste.rs",
    "src/terminal/rust/render_index.rs",
    "src/terminal/rust/render_row_data.rs",
    "src/terminal/rust/render_cell.rs",
    "src/terminal/rust/render_cell_style.rs",
    "src/terminal/rust/render_cell_text.rs",
    "src/terminal/rust/render_state.rs",
    "src/terminal/rust/render_state_primitive.rs",
    "src/terminal/rust/render_state_color.rs",
    "src/terminal/rust/render.rs",
    "src/terminal/rust/row.rs",
    "src/terminal/rust/selection.rs",
    "src/terminal/rust/selection_copy.rs",
    "src/terminal/rust/sgr_8color.rs",
    "src/terminal/rust/sgr_attr.rs",
    "src/terminal/rust/sgr_basic.rs",
    "src/terminal/rust/sgr_basic_write.rs",
    "src/terminal/rust/sgr_color.rs",
    "src/terminal/rust/sgr_constants.rs",
    "src/terminal/rust/sgr_parse.rs",
    "src/terminal/rust/sgr_state.rs",
    "src/terminal/rust/sgr_underline.rs",
    "src/terminal/rust/sgr_unknown.rs",
    "src/terminal/rust/sgr_write.rs",
    "src/terminal/rust/sgr.rs",
    "src/terminal/rust/simple.rs",
    "src/terminal/rust/size_report.rs",
    "src/terminal/rust/style.rs",
    "src/terminal/rust/style_copy.rs",
    "src/terminal/rust/style_write.rs",
    "src/terminal/rust/sys.rs",
    "src/terminal/rust/terminal_get.rs",
    "src/terminal/rust/terminal_get_color.rs",
    "src/terminal/rust/terminal_get_kitty_image.rs",
    "src/terminal/rust/terminal_get_payload.rs",
    "src/terminal/rust/terminal_get_pointer.rs",
    "src/terminal/rust/terminal_get_selection.rs",
    "src/terminal/rust/terminal_get_scalar.rs",
    "src/terminal/rust/terminal_set.rs",
    "src/terminal/rust/terminal_set_payload.rs",
    "src/terminal/rust/terminal_options.rs",
    "src/terminal/rust/terminal.rs",
};

fn addBuildInfoEnv(
    run: *std.Build.Step.Run,
    b: *std.Build,
    cfg: *const Config,
    options: TerminalBuildOptions,
) void {
    run.setEnvironmentVariable("GHOSTTY_VT_SIMD", if (options.simd) "1" else "0");
    run.setEnvironmentVariable("GHOSTTY_VT_KITTY_GRAPHICS", if (kittyGraphics(cfg)) "1" else "0");
    run.setEnvironmentVariable("GHOSTTY_VT_TMUX_CONTROL_MODE", if (options.oniguruma) "1" else "0");
    run.setEnvironmentVariable("GHOSTTY_VT_OPTIMIZE", switch (cfg.optimize) {
        .Debug => "debug",
        .ReleaseSafe => "release_safe",
        .ReleaseSmall => "release_small",
        .ReleaseFast => "release_fast",
    });
    run.setEnvironmentVariable("GHOSTTY_VT_VERSION_STRING", b.fmt("{f}", .{options.version}));
    run.setEnvironmentVariable("GHOSTTY_VT_VERSION_MAJOR", b.fmt("{d}", .{options.version.major}));
    run.setEnvironmentVariable("GHOSTTY_VT_VERSION_MINOR", b.fmt("{d}", .{options.version.minor}));
    run.setEnvironmentVariable("GHOSTTY_VT_VERSION_PATCH", b.fmt("{d}", .{options.version.patch}));
    run.setEnvironmentVariable("GHOSTTY_VT_VERSION_PRE", options.version.pre orelse "");
    run.setEnvironmentVariable("GHOSTTY_VT_VERSION_BUILD", options.version.build orelse "");
}

fn kittyGraphics(cfg: *const Config) bool {
    const target = cfg.target.result;
    return !(target.cpu.arch == .wasm32 and target.os.tag == .freestanding);
}

fn objectName(target: std.Target) []const u8 {
    return switch (target.ofmt) {
        .coff => "ghostty_vt_rust.obj",
        else => "ghostty_vt_rust.o",
    };
}

fn rustTargetTriple(target: std.Target) ?[]const u8 {
    return switch (target.os.tag) {
        .macos => switch (target.cpu.arch) {
            .aarch64 => "aarch64-apple-darwin",
            .x86_64 => "x86_64-apple-darwin",
            else => null,
        },
        .ios => switch (target.cpu.arch) {
            .aarch64 => if (target.abi == .simulator)
                "aarch64-apple-ios-sim"
            else
                "aarch64-apple-ios",
            .x86_64 => if (target.abi == .simulator)
                "x86_64-apple-ios"
            else
                null,
            else => null,
        },
        .linux => switch (target.cpu.arch) {
            .aarch64 => if (target.abi.isMusl())
                "aarch64-unknown-linux-musl"
            else
                "aarch64-unknown-linux-gnu",
            .x86_64 => if (target.abi.isMusl())
                "x86_64-unknown-linux-musl"
            else
                "x86_64-unknown-linux-gnu",
            else => null,
        },
        .windows => switch (target.cpu.arch) {
            .aarch64 => if (target.abi == .msvc)
                "aarch64-pc-windows-msvc"
            else
                null,
            .x86_64 => if (target.abi == .msvc)
                "x86_64-pc-windows-msvc"
            else
                "x86_64-pc-windows-gnu",
            else => null,
        },
        .freebsd => switch (target.cpu.arch) {
            .aarch64 => "aarch64-unknown-freebsd",
            .x86_64 => "x86_64-unknown-freebsd",
            else => null,
        },
        else => null,
    };
}
