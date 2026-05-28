const std = @import("std");
const testing = std.testing;
const lib = @import("../lib.zig");
const build_options = @import("terminal_options");
const PageList = @import("../PageList.zig");
const point = @import("../point.zig");
const grid_ref_c = @import("grid_ref.zig");
const terminal_c = @import("terminal.zig");
const Result = @import("result.zig").Result;

fn screenKeyByte(key: terminal_c.TerminalScreen) u8 {
    return switch (key) {
        .primary => 0,
        .alternate => 1,
    };
}

const rust_owned = if (build_options.terminal_rust_owned) struct {
    extern fn ghostty_rust_terminal_owned_tracked_page_list(
        handle: ?*anyopaque,
        screen_key: u8,
        generation: usize,
        out_pages: *?*PageList,
    ) callconv(.c) bool;

    extern fn ghostty_rust_tracked_pin_garbage(pin: *PageList.Pin) callconv(.c) bool;

    extern fn ghostty_rust_tracked_pin_to_grid_ref(
        pin: *PageList.Pin,
        out_ref: ?*grid_ref_c.CGridRef,
    ) callconv(.c) c_int;

    extern fn ghostty_rust_tracked_pin_point(
        pages: *PageList,
        tag: point.Tag,
        pin: *PageList.Pin,
        out: ?*point.Coordinate,
    ) callconv(.c) c_int;

    extern fn ghostty_rust_terminal_owned_tracked_grid_ref_set(
        handle: ?*anyopaque,
        pt: *const point.Point.C,
        old_pin: *PageList.Pin,
        out_pin: *?*PageList.Pin,
        out_screen_key: *u8,
        out_screen_generation: *usize,
    ) callconv(.c) c_int;

    extern fn ghostty_rust_terminal_owned_page_list_for_screen(
        handle: ?*anyopaque,
        screen_key: terminal_c.TerminalScreen,
        out_pages: *?*PageList,
    ) callconv(.c) bool;

    extern fn ghostty_rust_page_list_untrack_pin(
        pages: *PageList,
        pin: *PageList.Pin,
    ) callconv(.c) void;
} else struct {};

const rust = if (build_options.lib_vt_rust) struct {
    extern fn ghostty_rust_tracked_grid_ref_has_value(
        has_ref: bool,
        has_page_list: bool,
        garbage: bool,
    ) callconv(.c) bool;

    extern fn ghostty_rust_tracked_grid_ref_result(
        has_ref: bool,
        has_page_list: bool,
        garbage: bool,
        has_point: bool,
    ) callconv(.c) c_int;

    extern fn ghostty_rust_tracked_grid_ref_set_input(
        has_ref: bool,
        has_terminal: bool,
        same_terminal: bool,
    ) callconv(.c) c_int;
} else struct {};

/// C: GhosttyTrackedGridRef
///
/// An owned tracked reference to a position in the terminal grid. The
/// underlying PageList pin is automatically updated as the PageList changes.
pub const CTrackedGridRef = ?*TrackedGridRef;

pub const TrackedGridRef = struct {
    alloc: std.mem.Allocator,
    terminal: terminal_c.Terminal,
    screen_key: terminal_c.TerminalScreen,
    screen_generation: usize,
    pin: *PageList.Pin,

    /// Return the PageList that owns this tracked ref's pin, or null if the
    /// owning screen has been removed/reinitialized since the ref was created.
    fn pageList(ref: *const TrackedGridRef) ?*PageList {
        const wrapper = ref.terminal orelse return null;
        if (comptime build_options.terminal_rust_owned) {
            if (terminal_c.rustOwnedHandle(wrapper)) |handle| {
                var pages: ?*PageList = null;
                if (rust_owned.ghostty_rust_terminal_owned_tracked_page_list(
                    handle,
                    screenKeyByte(ref.screen_key),
                    ref.screen_generation,
                    &pages,
                )) return pages;
                return null;
            }
            return null;
        }
        const t = terminal_c.wrapperZig(wrapper) orelse return null;
        if (t.screens.generation(ref.screen_key) != ref.screen_generation) return null;
        const screen = t.screens.get(ref.screen_key) orelse return null;
        return &screen.pages;
    }

    fn pinGarbage(ref: *const TrackedGridRef) bool {
        if (comptime build_options.terminal_rust_owned) {
            if (ref.terminal) |wrapper| {
                if (terminal_c.rustOwnedHandle(wrapper) != null) {
                    return rust_owned.ghostty_rust_tracked_pin_garbage(ref.pin);
                }
            }
        }
        return ref.pin.garbage;
    }
};

pub fn tracked_grid_ref_free(ref_: CTrackedGridRef) callconv(lib.calling_conv) void {
    const ref = ref_ orelse return;
    if (ref.terminal) |wrapper| {
        _ = wrapper.tracked_grid_refs.swapRemove(ref);
    }
    if (comptime build_options.terminal_rust_owned) {
        if (ref.terminal) |wrapper| {
            if (terminal_c.rustOwnedHandle(wrapper)) |handle| {
                var pages: ?*PageList = null;
                if (rust_owned.ghostty_rust_terminal_owned_tracked_page_list(
                    handle,
                    screenKeyByte(ref.screen_key),
                    ref.screen_generation,
                    &pages,
                )) {
                    if (pages) |list| {
                        rust_owned.ghostty_rust_page_list_untrack_pin(list, ref.pin);
                    }
                }
            } else if (ref.pageList()) |list| {
                list.untrackPin(ref.pin);
            }
        }
    } else if (ref.pageList()) |list| {
        list.untrackPin(ref.pin);
    }
    ref.alloc.destroy(ref);
}

pub fn tracked_grid_ref_has_value(ref_: CTrackedGridRef) callconv(lib.calling_conv) bool {
    const ref = ref_ orelse return false;
    const has_page_list = ref.pageList() != null;
    const garbage = if (has_page_list) ref.pinGarbage() else true;
    if (comptime build_options.lib_vt_rust) {
        return rust.ghostty_rust_tracked_grid_ref_has_value(
            true,
            has_page_list,
            garbage,
        );
    }

    if (!has_page_list) return false;
    return !garbage;
}

pub fn tracked_grid_ref_snapshot(
    ref_: CTrackedGridRef,
    out_ref: ?*grid_ref_c.CGridRef,
) callconv(lib.calling_conv) Result {
    const ref = ref_ orelse return .invalid_value;
    const has_page_list = ref.pageList() != null;
    const garbage = if (has_page_list) ref.pinGarbage() else true;
    if (comptime build_options.lib_vt_rust) {
        const result: Result = @enumFromInt(rust.ghostty_rust_tracked_grid_ref_result(
            true,
            has_page_list,
            garbage,
            true,
        ));
        if (result != .success) return result;
    } else {
        if (!has_page_list) return .no_value;
        if (garbage) return .no_value;
    }

    if (out_ref) |out| {
        if (comptime build_options.terminal_rust_owned) {
            if (ref.terminal) |wrapper| {
                if (terminal_c.rustOwnedHandle(wrapper) != null) {
                    return @enumFromInt(rust_owned.ghostty_rust_tracked_pin_to_grid_ref(
                        ref.pin,
                        out,
                    ));
                }
            }
        }
        out.* = grid_ref_c.CGridRef.fromPin(ref.pin.*);
    }
    return .success;
}

pub fn tracked_grid_ref_point(
    ref_: CTrackedGridRef,
    tag: point.Tag,
    out: ?*point.Coordinate,
) callconv(lib.calling_conv) Result {
    const ref = ref_ orelse return .invalid_value;
    const list = ref.pageList();
    const garbage = if (list != null) ref.pinGarbage() else true;
    var pt: ?point.Point = null;
    if (list) |l| {
        if (!garbage) {
            if (comptime build_options.terminal_rust_owned) {
                if (ref.terminal) |wrapper| {
                    if (terminal_c.rustOwnedHandle(wrapper) != null) {
                        var coord: point.Coordinate = undefined;
                        const result: Result = @enumFromInt(rust_owned.ghostty_rust_tracked_pin_point(
                            l,
                            tag,
                            ref.pin,
                            &coord,
                        ));
                        if (result == .success) {
                            if (out) |o| o.* = coord;
                            return .success;
                        }
                        return result;
                    }
                }
            }
            pt = l.pointFromPin(tag, ref.pin.*);
        }
    }

    if (comptime build_options.lib_vt_rust) {
        const result: Result = @enumFromInt(rust.ghostty_rust_tracked_grid_ref_result(
            true,
            list != null,
            garbage,
            pt != null,
        ));
        if (result != .success) return result;
    } else {
        _ = list orelse return .no_value;
        if (garbage) return .no_value;
        _ = pt orelse return .no_value;
    }

    if (out) |o| o.* = pt.?.coord();
    return .success;
}

pub fn tracked_grid_ref_set(
    ref_: CTrackedGridRef,
    terminal_: terminal_c.Terminal,
    pt: point.Point.C,
) callconv(lib.calling_conv) Result {
    if (comptime build_options.lib_vt_rust) {
        const same_terminal = if (ref_ != null and terminal_ != null)
            ref_.?.terminal == terminal_
        else
            false;
        const result: Result = @enumFromInt(rust.ghostty_rust_tracked_grid_ref_set_input(
            ref_ != null,
            terminal_ != null,
            same_terminal,
        ));
        if (result != .success) return result;
    }

    const ref = ref_ orelse return .invalid_value;
    const wrapper = terminal_ orelse return .invalid_value;
    if (ref.terminal != terminal_) return .invalid_value;

    if (comptime build_options.terminal_rust_owned) {
        if (terminal_c.rustOwnedHandle(wrapper)) |handle| {
            var old_pages: ?*PageList = null;
            if (rust_owned.ghostty_rust_terminal_owned_tracked_page_list(
                handle,
                screenKeyByte(ref.screen_key),
                ref.screen_generation,
                &old_pages,
            )) {
                if (old_pages) |pages| {
                    rust_owned.ghostty_rust_page_list_untrack_pin(pages, ref.pin);
                }
            }

            var tracked_pin: ?*PageList.Pin = null;
            var screen_key_byte: u8 = 0;
            var screen_generation: usize = 0;
            const result: Result = @enumFromInt(rust_owned.ghostty_rust_terminal_owned_tracked_grid_ref_set(
                handle,
                &pt,
                ref.pin,
                &tracked_pin,
                &screen_key_byte,
                &screen_generation,
            ));
            if (result != .success) return result;
            ref.screen_key = @enumFromInt(screen_key_byte);
            ref.screen_generation = screen_generation;
            ref.pin = tracked_pin.?;
            return .success;
        }
    }

    const t = terminal_c.wrapperZig(wrapper) orelse return .invalid_value;
    const list = &t.screens.active.pages;
    const p = list.pin(point.Point.fromC(pt)) orelse return .invalid_value;
    const tracked_pin = list.trackPin(p) catch return .out_of_memory;

    if (ref.pageList()) |old_list| old_list.untrackPin(ref.pin);
    ref.screen_key = t.screens.active_key;
    ref.screen_generation = t.screens.generation(ref.screen_key);
    ref.pin = tracked_pin;
    return .success;
}

test "tracked_grid_ref snapshots after terminal scroll" {
    var terminal: terminal_c.Terminal = null;
    try testing.expectEqual(Result.success, terminal_c.new(
        &lib.alloc.test_allocator,
        &terminal,
        .{ .cols = 5, .rows = 2, .max_scrollback = 10_000 },
    ));
    defer terminal_c.free(terminal);

    terminal_c.vt_write(terminal, "A", 1);

    var ref: CTrackedGridRef = null;
    try testing.expectEqual(Result.success, terminal_c.grid_ref_track(
        terminal,
        point.Point.cval(.{ .active = .{ .x = 0, .y = 0 } }),
        &ref,
    ));
    defer tracked_grid_ref_free(ref);

    terminal_c.vt_write(terminal, "\r\nB\r\nC", 6);
    try testing.expect(tracked_grid_ref_has_value(ref));

    var snapshot: grid_ref_c.CGridRef = undefined;
    try testing.expectEqual(Result.success, tracked_grid_ref_snapshot(ref, &snapshot));

    var buf: [1]u32 = undefined;
    var len: usize = undefined;
    try testing.expectEqual(Result.success, grid_ref_c.grid_ref_graphemes(&snapshot, &buf, buf.len, &len));
    try testing.expectEqual(@as(usize, 1), len);
    try testing.expectEqual(@as(u32, 'A'), buf[0]);
}

test "tracked_grid_ref reports no value after reset" {
    var terminal: terminal_c.Terminal = null;
    try testing.expectEqual(Result.success, terminal_c.new(
        &lib.alloc.test_allocator,
        &terminal,
        .{ .cols = 5, .rows = 2, .max_scrollback = 10_000 },
    ));
    defer terminal_c.free(terminal);

    terminal_c.vt_write(terminal, "A", 1);

    var ref: CTrackedGridRef = null;
    try testing.expectEqual(Result.success, terminal_c.grid_ref_track(
        terminal,
        point.Point.cval(.{ .active = .{ .x = 0, .y = 0 } }),
        &ref,
    ));
    defer tracked_grid_ref_free(ref);

    terminal_c.reset(terminal);
    try testing.expect(!tracked_grid_ref_has_value(ref));

    var snapshot: grid_ref_c.CGridRef = undefined;
    try testing.expectEqual(Result.no_value, tracked_grid_ref_snapshot(ref, &snapshot));
}

test "tracked_grid_ref reports no value after alternate screen reset" {
    var terminal: terminal_c.Terminal = null;
    try testing.expectEqual(Result.success, terminal_c.new(
        &lib.alloc.test_allocator,
        &terminal,
        .{ .cols = 5, .rows = 2, .max_scrollback = 10_000 },
    ));
    defer terminal_c.free(terminal);

    terminal_c.vt_write(terminal, "\x1b[?1049hA", 9);

    var ref: CTrackedGridRef = null;
    try testing.expectEqual(Result.success, terminal_c.grid_ref_track(
        terminal,
        point.Point.cval(.{ .active = .{ .x = 0, .y = 0 } }),
        &ref,
    ));
    defer tracked_grid_ref_free(ref);

    terminal_c.vt_write(terminal, "\x1bc", 2);
    try testing.expect(!tracked_grid_ref_has_value(ref));

    var snapshot: grid_ref_c.CGridRef = undefined;
    try testing.expectEqual(Result.no_value, tracked_grid_ref_snapshot(ref, &snapshot));

    var coord: point.Coordinate = undefined;
    try testing.expectEqual(Result.no_value, tracked_grid_ref_point(ref, .active, &coord));

    terminal_c.vt_write(terminal, "\x1b[?1049h", 8);
    try testing.expect(!tracked_grid_ref_has_value(ref));

    try testing.expectEqual(Result.success, tracked_grid_ref_set(
        ref,
        terminal,
        point.Point.cval(.{ .active = .{ .x = 0, .y = 0 } }),
    ));
    try testing.expect(tracked_grid_ref_has_value(ref));
    try testing.expectEqual(Result.success, tracked_grid_ref_snapshot(ref, &snapshot));
}

test "tracked_grid_ref reports no value after terminal free" {
    var terminal: terminal_c.Terminal = null;
    try testing.expectEqual(Result.success, terminal_c.new(
        &lib.alloc.test_allocator,
        &terminal,
        .{ .cols = 5, .rows = 2, .max_scrollback = 10_000 },
    ));

    terminal_c.vt_write(terminal, "A", 1);

    var ref: CTrackedGridRef = null;
    try testing.expectEqual(Result.success, terminal_c.grid_ref_track(
        terminal,
        point.Point.cval(.{ .active = .{ .x = 0, .y = 0 } }),
        &ref,
    ));

    terminal_c.free(terminal);
    try testing.expect(!tracked_grid_ref_has_value(ref));

    var snapshot: grid_ref_c.CGridRef = undefined;
    try testing.expectEqual(Result.no_value, tracked_grid_ref_snapshot(ref, &snapshot));

    var coord: point.Coordinate = undefined;
    try testing.expectEqual(Result.no_value, tracked_grid_ref_point(ref, .active, &coord));

    try testing.expectEqual(Result.invalid_value, tracked_grid_ref_set(
        ref,
        terminal,
        point.Point.cval(.{ .active = .{ .x = 0, .y = 0 } }),
    ));

    tracked_grid_ref_free(ref);
}

test "tracked_grid_ref null input" {
    try testing.expect(!tracked_grid_ref_has_value(null));

    var snapshot: grid_ref_c.CGridRef = undefined;
    try testing.expectEqual(Result.invalid_value, tracked_grid_ref_snapshot(null, &snapshot));

    var coord: point.Coordinate = undefined;
    try testing.expectEqual(Result.invalid_value, tracked_grid_ref_point(null, .active, &coord));

    try testing.expectEqual(Result.invalid_value, tracked_grid_ref_set(
        null,
        null,
        point.Point.cval(.{ .active = .{ .x = 0, .y = 0 } }),
    ));
}

test "tracked_grid_ref set rejects other terminal" {
    var terminal_a: terminal_c.Terminal = null;
    try testing.expectEqual(Result.success, terminal_c.new(
        &lib.alloc.test_allocator,
        &terminal_a,
        .{ .cols = 5, .rows = 2, .max_scrollback = 10_000 },
    ));
    defer terminal_c.free(terminal_a);

    var terminal_b: terminal_c.Terminal = null;
    try testing.expectEqual(Result.success, terminal_c.new(
        &lib.alloc.test_allocator,
        &terminal_b,
        .{ .cols = 5, .rows = 2, .max_scrollback = 10_000 },
    ));
    defer terminal_c.free(terminal_b);

    terminal_c.vt_write(terminal_a, "A", 1);

    var ref: CTrackedGridRef = null;
    try testing.expectEqual(Result.success, terminal_c.grid_ref_track(
        terminal_a,
        point.Point.cval(.{ .active = .{ .x = 0, .y = 0 } }),
        &ref,
    ));
    defer tracked_grid_ref_free(ref);

    try testing.expectEqual(Result.invalid_value, tracked_grid_ref_set(
        ref,
        terminal_b,
        point.Point.cval(.{ .active = .{ .x = 0, .y = 0 } }),
    ));
}
