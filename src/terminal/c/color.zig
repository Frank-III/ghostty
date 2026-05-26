const lib = @import("../lib.zig");
const build_options = @import("terminal_options");
const color = @import("../color.zig");

const rust = if (build_options.lib_vt_rust) struct {
    extern fn ghostty_rust_color_rgb_get(
        c: color.RGB.C,
        r: *u8,
        g: *u8,
        b: *u8,
    ) callconv(.c) void;
} else struct {};

pub fn rgb_get(
    c: color.RGB.C,
    r: *u8,
    g: *u8,
    b: *u8,
) callconv(lib.calling_conv) void {
    if (comptime build_options.lib_vt_rust) {
        return rust.ghostty_rust_color_rgb_get(c, r, g, b);
    }

    r.* = c.r;
    g.* = c.g;
    b.* = c.b;
}
