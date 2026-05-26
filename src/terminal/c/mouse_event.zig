const std = @import("std");
const Allocator = std.mem.Allocator;
const testing = std.testing;
const lib = @import("../lib.zig");
const build_options = @import("terminal_options");
const CAllocator = lib.alloc.Allocator;
const key = @import("../../input/key.zig");
const mouse = @import("../../input/mouse.zig");
const mouse_encode = @import("../../input/mouse_encode.zig");
const Result = @import("result.zig").Result;

const log = std.log.scoped(.mouse_event);

/// Wrapper around mouse event that tracks the allocator for C API usage.
const MouseEventWrapper = struct {
    event: mouse_encode.Event = .{},
    alloc: Allocator,
};

/// C: GhosttyMouseEvent
pub const Event = ?*MouseEventWrapper;

/// C: GhosttyMouseAction
pub const Action = mouse.Action;

/// C: GhosttyMouseButton
pub const Button = mouse.Button;

/// C: GhosttyMousePosition
pub const Position = mouse_encode.Event.Pos;

/// C: GhosttyMods
pub const Mods = key.Mods;

const rust = if (build_options.lib_vt_rust) struct {
    extern fn ghostty_rust_mouse_event_set_action(event: *anyopaque, action: c_int) callconv(.c) void;
    extern fn ghostty_rust_mouse_event_get_action(event: *anyopaque) callconv(.c) c_int;
    extern fn ghostty_rust_mouse_event_set_button(event: *anyopaque, button: c_int) callconv(.c) void;
    extern fn ghostty_rust_mouse_event_clear_button(event: *anyopaque) callconv(.c) void;
    extern fn ghostty_rust_mouse_event_get_button(event: *anyopaque, out: ?*Button) callconv(.c) bool;
    extern fn ghostty_rust_mouse_event_set_mods(event: *anyopaque, mods: u16) callconv(.c) void;
    extern fn ghostty_rust_mouse_event_get_mods(event: *anyopaque) callconv(.c) u16;
    extern fn ghostty_rust_mouse_event_set_position(event: *anyopaque, pos: Position) callconv(.c) void;
    extern fn ghostty_rust_mouse_event_get_position(event: *anyopaque) callconv(.c) Position;
} else struct {};

pub fn new(
    alloc_: ?*const CAllocator,
    result: *Event,
) callconv(lib.calling_conv) Result {
    const alloc = lib.alloc.default(alloc_);
    const ptr = alloc.create(MouseEventWrapper) catch
        return .out_of_memory;
    ptr.* = .{ .alloc = alloc };
    result.* = ptr;
    return .success;
}

pub fn free(event_: Event) callconv(lib.calling_conv) void {
    const wrapper = event_ orelse return;
    const alloc = wrapper.alloc;
    alloc.destroy(wrapper);
}

pub fn set_action(event_: Event, action: Action) callconv(lib.calling_conv) void {
    if (comptime std.debug.runtime_safety) {
        _ = std.meta.intToEnum(Action, @intFromEnum(action)) catch {
            log.warn("set_action invalid action value={d}", .{@intFromEnum(action)});
            return;
        };
    }

    if (comptime build_options.lib_vt_rust) {
        return rust.ghostty_rust_mouse_event_set_action(event_.?, @intFromEnum(action));
    }

    event_.?.event.action = action;
}

pub fn get_action(event_: Event) callconv(lib.calling_conv) Action {
    if (comptime build_options.lib_vt_rust) {
        return @enumFromInt(rust.ghostty_rust_mouse_event_get_action(event_.?));
    }

    return event_.?.event.action;
}

pub fn set_button(event_: Event, button: Button) callconv(lib.calling_conv) void {
    if (comptime std.debug.runtime_safety) {
        _ = std.meta.intToEnum(Button, @intFromEnum(button)) catch {
            log.warn("set_button invalid button value={d}", .{@intFromEnum(button)});
            return;
        };
    }

    if (comptime build_options.lib_vt_rust) {
        return rust.ghostty_rust_mouse_event_set_button(event_.?, @intFromEnum(button));
    }

    event_.?.event.button = button;
}

pub fn clear_button(event_: Event) callconv(lib.calling_conv) void {
    if (comptime build_options.lib_vt_rust) {
        return rust.ghostty_rust_mouse_event_clear_button(event_.?);
    }

    event_.?.event.button = null;
}

pub fn get_button(event_: Event, out: ?*Button) callconv(lib.calling_conv) bool {
    if (comptime build_options.lib_vt_rust) {
        return rust.ghostty_rust_mouse_event_get_button(event_.?, out);
    }

    if (event_.?.event.button) |button| {
        if (out) |ptr| ptr.* = button;
        return true;
    }

    return false;
}

pub fn set_mods(event_: Event, mods: Mods) callconv(lib.calling_conv) void {
    if (comptime build_options.lib_vt_rust) {
        return rust.ghostty_rust_mouse_event_set_mods(event_.?, @bitCast(mods));
    }

    event_.?.event.mods = mods;
}

pub fn get_mods(event_: Event) callconv(lib.calling_conv) Mods {
    if (comptime build_options.lib_vt_rust) {
        return @bitCast(rust.ghostty_rust_mouse_event_get_mods(event_.?));
    }

    return event_.?.event.mods;
}

pub fn set_position(event_: Event, pos: Position) callconv(lib.calling_conv) void {
    if (comptime build_options.lib_vt_rust) {
        return rust.ghostty_rust_mouse_event_set_position(event_.?, pos);
    }

    event_.?.event.pos = pos;
}

pub fn get_position(event_: Event) callconv(lib.calling_conv) Position {
    if (comptime build_options.lib_vt_rust) {
        return rust.ghostty_rust_mouse_event_get_position(event_.?);
    }

    return event_.?.event.pos;
}

test "alloc" {
    var e: Event = undefined;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &e,
    ));
    free(e);
}

test "free null" {
    free(null);
}

test "rust layout offsets" {
    const no_button: ?mouse.Button = null;
    const unknown_button: ?mouse.Button = .unknown;
    const left_button: ?mouse.Button = .left;

    try testing.expectEqual(@as(usize, 16), @offsetOf(MouseEventWrapper, "event"));
    try testing.expectEqual(@as(usize, 0), @offsetOf(mouse_encode.Event, "action"));
    try testing.expectEqual(@as(usize, 4), @offsetOf(mouse_encode.Event, "button"));
    try testing.expectEqual(@as(usize, 12), @offsetOf(mouse_encode.Event, "pos"));
    try testing.expectEqual(@as(usize, 20), @offsetOf(mouse_encode.Event, "mods"));
    try testing.expectEqual(@as(usize, 24), @sizeOf(mouse_encode.Event));
    try testing.expectEqual(@as(usize, 8), @sizeOf(?mouse.Button));
    try testing.expectEqual(@as(usize, 8), @sizeOf(Position));
    try testing.expectEqual(
        @as(u64, 0),
        std.mem.readInt(u64, std.mem.asBytes(&no_button)[0..8], .little),
    );
    try testing.expectEqual(
        @as(u64, 0x1_00000000),
        std.mem.readInt(u64, std.mem.asBytes(&unknown_button)[0..8], .little),
    );
    try testing.expectEqual(
        @as(u64, 0x1_00000001),
        std.mem.readInt(u64, std.mem.asBytes(&left_button)[0..8], .little),
    );
}

test "set/get" {
    var e: Event = undefined;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &e,
    ));
    defer free(e);

    // Action
    set_action(e, .motion);
    try testing.expectEqual(Action.motion, get_action(e));

    // Button
    set_button(e, .left);
    var button: Button = .unknown;
    try testing.expect(get_button(e, &button));
    try testing.expectEqual(Button.left, button);
    try testing.expect(get_button(e, null));

    set_button(e, .unknown);
    try testing.expect(get_button(e, &button));
    try testing.expectEqual(Button.unknown, button);

    clear_button(e);
    try testing.expect(!get_button(e, &button));

    // Mods
    const mods: Mods = .{ .shift = true, .ctrl = true };
    set_mods(e, mods);
    const got_mods = get_mods(e);
    try testing.expect(got_mods.shift);
    try testing.expect(got_mods.ctrl);
    try testing.expect(!got_mods.alt);

    // Position
    set_position(e, .{ .x = 12.5, .y = -4.0 });
    const pos = get_position(e);
    try testing.expectEqual(@as(f32, 12.5), pos.x);
    try testing.expectEqual(@as(f32, -4.0), pos.y);
}
