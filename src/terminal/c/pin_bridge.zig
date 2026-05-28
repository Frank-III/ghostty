const std = @import("std");
const builtin = @import("builtin");
const Allocator = std.mem.Allocator;
const lib = @import("../lib.zig");
const CAllocator = lib.alloc.Allocator;
const PageList = @import("../PageList.zig");
const MemoryPool = PageList.MemoryPool;
const Pin = PageList.Pin;

const page_preheat: usize = 4;

fn pageAllocator() Allocator {
    if (builtin.target.os.tag.isDarwin()) {
        const mach = @import("../../os/mach.zig");
        return mach.taggedPageAllocator(.application_specific_1);
    }
    return std.heap.page_allocator;
}

/// Create a PageList memory pool backed by the given C allocator.
pub fn memoryPoolCreate(
    alloc_: ?*const CAllocator,
    preheat: usize,
) callconv(lib.calling_conv) ?*anyopaque {
    const alloc = lib.alloc.default(alloc_);
    const pool = alloc.create(MemoryPool) catch return null;
    pool.* = MemoryPool.init(
        alloc,
        pageAllocator(),
        preheat,
    ) catch {
        alloc.destroy(pool);
        return null;
    };
    return pool;
}

/// Destroy a memory pool created by memoryPoolCreate.
pub fn memoryPoolDestroy(
    alloc_: ?*const CAllocator,
    pool_ptr: ?*anyopaque,
) callconv(lib.calling_conv) void {
    const pool = pool_ptr orelse return;
    const alloc = lib.alloc.default(alloc_);
    const p: *MemoryPool = @ptrCast(@alignCast(pool));
    p.deinit();
    alloc.destroy(p);
}

/// Allocate a page-list node from the pool.
pub fn poolCreateNode(pool_ptr: ?*anyopaque) callconv(lib.calling_conv) ?*anyopaque {
    const pool = pool_ptr orelse return null;
    const pool_ref: *MemoryPool = @ptrCast(@alignCast(pool));
    return pool_ref.nodes.create() catch null;
}

/// Return a page-list node to the pool.
pub fn poolDestroyNode(
    pool_ptr: ?*anyopaque,
    node_ptr: ?*anyopaque,
) callconv(lib.calling_conv) void {
    const pool = pool_ptr orelse return;
    const node = node_ptr orelse return;
    const pool_ref: *MemoryPool = @ptrCast(@alignCast(pool));
    const node_ref: *PageList.List.Node = @ptrCast(@alignCast(node));
    pool_ref.nodes.destroy(node_ref);
}

/// Allocate a standard-sized page buffer from the pool.
pub fn poolCreateStdPage(pool_ptr: ?*anyopaque) callconv(lib.calling_conv) ?[*]u8 {
    const pool = pool_ptr orelse return null;
    const pool_ref: *MemoryPool = @ptrCast(@alignCast(pool));
    const page = pool_ref.pages.create() catch return null;
    return page.ptr;
}

/// Return a standard-sized page buffer to the pool.
pub fn poolDestroyStdPage(
    pool_ptr: ?*anyopaque,
    page: ?[*]u8,
) callconv(lib.calling_conv) void {
    const pool = pool_ptr orelse return;
    const page_ptr = page orelse return;
    const pool_ref: *MemoryPool = @ptrCast(@alignCast(pool));
    const page_pkg = @import("../page.zig");
    const std_size = comptime page_pkg.Page.layout(page_pkg.std_capacity).total_size;
    const page_aligned: *align(std.heap.page_size_min) [std_size]u8 = @ptrCast(@alignCast(page_ptr));
    pool_ref.pages.destroy(page_aligned);
}

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
    _ = memoryPoolCreate;
    _ = memoryPoolDestroy;
    _ = poolCreateNode;
    _ = poolDestroyNode;
    _ = poolCreateStdPage;
    _ = poolDestroyStdPage;
}
