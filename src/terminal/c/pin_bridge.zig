const std = @import("std");
const Allocator = std.mem.Allocator;
const lib = @import("../lib.zig");
const PageList = @import("../PageList.zig");
const MemoryPool = PageList.MemoryPool;
const Pin = PageList.Pin;

/// Allocate a Pin from the pool and write the given fields.
/// Returns an opaque pointer to the allocated Pin, or null on failure.
pub fn pinCreate(
    pool_ptr: *anyopaque,
    node: ?*anyopaque,
    y: u16,
    x: u16,
    garbage: bool,
) callconv(lib.calling_conv) ?*anyopaque {
    const pool: *MemoryPool = @ptrCast(@alignCast(pool_ptr));
    const p = pool.pins.create() catch return null;
    p.* = .{
        .node = @ptrCast(@alignCast(node)),
        .y = y,
        .x = x,
        .garbage = garbage,
    };
    return p;
}

/// Return a Pin (opaque pointer) to the pool.
pub fn pinDestroy(
    pool_ptr: *anyopaque,
    pin: *anyopaque,
) callconv(lib.calling_conv) void {
    const pool: *MemoryPool = @ptrCast(@alignCast(pool_ptr));
    const p: *Pin = @ptrCast(@alignCast(pin));
    pool.pins.destroy(p);
}

/// General-purpose allocation from the pool's allocator.
/// Used for growing the tracked-pins keys array in Rust.
pub fn poolAlloc(
    pool_ptr: *anyopaque,
    size: usize,
) callconv(lib.calling_conv) ?[*]u8 {
    const pool: *MemoryPool = @ptrCast(@alignCast(pool_ptr));
    const mem = pool.alloc.alloc(u8, size) catch return null;
    return mem.ptr;
}

/// General-purpose free from the pool's allocator.
pub fn poolFree(
    pool_ptr: *anyopaque,
    ptr: [*]u8,
    size: usize,
) callconv(lib.calling_conv) void {
    const pool: *MemoryPool = @ptrCast(@alignCast(pool_ptr));
    pool.alloc.free(ptr[0..size]);
}

test {
    _ = pinCreate;
    _ = pinDestroy;
    _ = poolAlloc;
    _ = poolFree;
}
