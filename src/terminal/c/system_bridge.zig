const std = @import("std");
const builtin = @import("builtin");
const lib = @import("../lib.zig");
const terminal_sys = @import("../sys.zig");

pub const Image = extern struct {
    width: u32 = 0,
    height: u32 = 0,
    data: ?[*]u8 = null,
    data_len: usize = 0,
};

pub fn systemPngAvailable() callconv(lib.calling_conv) bool {
    return terminal_sys.decode_png != null;
}

pub fn systemDecodePng(
    data: [*]const u8,
    data_len: usize,
    buf: [*]u8,
    buf_cap: usize,
    out: *Image,
) callconv(lib.calling_conv) bool {
    const decode_fn = terminal_sys.decode_png orelse return false;

    var fba = std.heap.FixedBufferAllocator.init(buf[0..buf_cap]);
    const alloc = fba.allocator();

    const image = decode_fn(alloc, data[0..data_len]) catch return false;

    out.width = image.width;
    out.height = image.height;
    out.data = image.data.ptr;
    out.data_len = image.data.len;
    return true;
}

test "systemPngAvailable when no decoder installed" {
    const prev = terminal_sys.decode_png;
    defer terminal_sys.decode_png = prev;

    terminal_sys.decode_png = null;
    try std.testing.expect(!systemPngAvailable());
}
