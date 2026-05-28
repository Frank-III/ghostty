const std = @import("std");
const Allocator = std.mem.Allocator;
const testing = std.testing;
const build_options = @import("terminal_options");
const lib = @import("../lib.zig");
const CAllocator = lib.alloc.Allocator;
const input_mouse_encode = @import("../../input/mouse_encode.zig");
const renderer_size = @import("../../renderer/size.zig");
const point = @import("../point.zig");
const terminal_mouse = @import("../mouse.zig");
const mouse_event = @import("mouse_event.zig");
const Result = @import("result.zig").Result;
const Event = mouse_event.Event;
const Terminal = @import("terminal.zig").Terminal;
const ZigTerminal = @import("../Terminal.zig");

const log = std.log.scoped(.mouse_encode);

const rust = if (build_options.lib_vt_rust) struct {
    extern fn ghostty_rust_mouse_encoder_setopt_event(
        value: c_int,
        current: c_int,
        out: *TrackingMode,
        last_cell_present: *bool,
    ) callconv(.c) void;

    extern fn ghostty_rust_mouse_encoder_setopt_format(
        value: c_int,
        current: c_int,
        out: *Format,
        last_cell_present: *bool,
    ) callconv(.c) void;

    extern fn ghostty_rust_mouse_encoder_setopt_size(
        screen_width: u32,
        screen_height: u32,
        cell_width: u32,
        cell_height: u32,
        padding_top: u32,
        padding_bottom: u32,
        padding_right: u32,
        padding_left: u32,
        out_screen_width: *u32,
        out_screen_height: *u32,
        out_cell_width: *u32,
        out_cell_height: *u32,
        out_padding_top: *u32,
        out_padding_bottom: *u32,
        out_padding_right: *u32,
        out_padding_left: *u32,
        last_cell_present: *bool,
    ) callconv(.c) void;

    extern fn ghostty_rust_mouse_encoder_setopt_bool(
        option: c_int,
        value: bool,
        any_button_pressed: *bool,
        track_last_cell: *bool,
        last_cell_present: *bool,
    ) callconv(.c) void;

    extern fn ghostty_rust_mouse_encoder_from_terminal(
        event: c_int,
        format: c_int,
        out_event: *TrackingMode,
        out_format: *Format,
        last_cell_present: *bool,
    ) callconv(.c) void;

    extern fn ghostty_rust_mouse_encoder_reset(
        last_cell_present: *bool,
    ) callconv(.c) void;

    extern fn ghostty_rust_mouse_encode(
        action: c_int,
        button_present: bool,
        button: c_int,
        mods: u16,
        pos: mouse_event.Position,
        tracking_mode: c_int,
        format: c_int,
        screen_width: u32,
        screen_height: u32,
        cell_width: u32,
        cell_height: u32,
        padding_top: u32,
        padding_bottom: u32,
        padding_right: u32,
        padding_left: u32,
        any_button_pressed: bool,
        track_last_cell: bool,
        last_cell_present: bool,
        last_cell_x: u16,
        last_cell_y: u32,
        out: ?[*]u8,
        out_len: usize,
        out_written: *usize,
        next_last_cell_present: *bool,
        next_last_cell_x: *u16,
        next_last_cell_y: *u32,
    ) callconv(.c) c_int;
} else struct {};

/// Wrapper around mouse encoding options that tracks the allocator for C API usage.
const MouseEncoderWrapper = struct {
    opts: input_mouse_encode.Options,
    track_last_cell: bool = false,
    last_cell: ?point.Coordinate = null,
    alloc: Allocator,
};

/// C: GhosttyMouseEncoder
pub const Encoder = ?*MouseEncoderWrapper;

/// C: GhosttyMouseTrackingMode
pub const TrackingMode = terminal_mouse.Event;

/// C: GhosttyMouseFormat
pub const Format = terminal_mouse.Format;

/// C: GhosttyMouseEncoderSize
pub const Size = extern struct {
    size: usize = @sizeOf(Size),
    screen_width: u32,
    screen_height: u32,
    cell_width: u32,
    cell_height: u32,
    padding_top: u32,
    padding_bottom: u32,
    padding_right: u32,
    padding_left: u32,

    fn toRenderer(self: Size) ?renderer_size.Size {
        if (self.cell_width == 0 or self.cell_height == 0) return null;
        return .{
            .screen = .{
                .width = self.screen_width,
                .height = self.screen_height,
            },
            .cell = .{
                .width = self.cell_width,
                .height = self.cell_height,
            },
            .padding = .{
                .top = self.padding_top,
                .bottom = self.padding_bottom,
                .right = self.padding_right,
                .left = self.padding_left,
            },
        };
    }
};

/// C: GhosttyMouseEncoderOption
pub const Option = enum(c_int) {
    event = 0,
    format = 1,
    size = 2,
    any_button_pressed = 3,
    track_last_cell = 4,

    /// Input type expected for setting the option.
    pub fn InType(comptime self: Option) type {
        return switch (self) {
            .event => TrackingMode,
            .format => Format,
            .size => Size,
            .any_button_pressed,
            .track_last_cell,
            => bool,
        };
    }
};

pub fn new(
    alloc_: ?*const CAllocator,
    result: *Encoder,
) callconv(lib.calling_conv) Result {
    const alloc = lib.alloc.default(alloc_);
    const ptr = alloc.create(MouseEncoderWrapper) catch
        return .out_of_memory;
    ptr.* = .{
        .opts = .{ .size = defaultSize() },
        .alloc = alloc,
    };
    result.* = ptr;
    return .success;
}

pub fn free(encoder_: Encoder) callconv(lib.calling_conv) void {
    const wrapper = encoder_ orelse return;
    const alloc = wrapper.alloc;
    alloc.destroy(wrapper);
}

pub fn setopt(
    encoder_: Encoder,
    option: Option,
    value: ?*const anyopaque,
) callconv(lib.calling_conv) void {
    if (comptime std.debug.runtime_safety) {
        _ = std.meta.intToEnum(Option, @intFromEnum(option)) catch {
            log.warn("setopt invalid option value={d}", .{@intFromEnum(option)});
            return;
        };
    }

    return switch (option) {
        inline else => |comptime_option| setoptTyped(
            encoder_,
            comptime_option,
            @ptrCast(@alignCast(value orelse return)),
        ),
    };
}

fn setoptTyped(
    encoder_: Encoder,
    comptime option: Option,
    value: *const option.InType(),
) void {
    const wrapper = encoder_.?;
    switch (option) {
        .event => {
            if (comptime std.debug.runtime_safety) {
                _ = std.meta.intToEnum(TrackingMode, @intFromEnum(value.*)) catch {
                    log.warn("setopt invalid TrackingMode value={d}", .{@intFromEnum(value.*)});
                    return;
                };
            }

            if (comptime build_options.lib_vt_rust) {
                var last_cell_present = wrapper.last_cell != null;
                rust.ghostty_rust_mouse_encoder_setopt_event(
                    @intFromEnum(value.*),
                    @intFromEnum(wrapper.opts.event),
                    &wrapper.opts.event,
                    &last_cell_present,
                );
                if (!last_cell_present) wrapper.last_cell = null;
            } else {
                if (wrapper.opts.event != value.*) wrapper.last_cell = null;
                wrapper.opts.event = value.*;
            }
        },

        .format => {
            if (comptime std.debug.runtime_safety) {
                _ = std.meta.intToEnum(Format, @intFromEnum(value.*)) catch {
                    log.warn("setopt invalid Format value={d}", .{@intFromEnum(value.*)});
                    return;
                };
            }

            if (comptime build_options.lib_vt_rust) {
                var last_cell_present = wrapper.last_cell != null;
                rust.ghostty_rust_mouse_encoder_setopt_format(
                    @intFromEnum(value.*),
                    @intFromEnum(wrapper.opts.format),
                    &wrapper.opts.format,
                    &last_cell_present,
                );
                if (!last_cell_present) wrapper.last_cell = null;
            } else {
                if (wrapper.opts.format != value.*) wrapper.last_cell = null;
                wrapper.opts.format = value.*;
            }
        },

        .size => {
            if (value.size < @sizeOf(Size)) {
                log.warn("setopt size struct too small size={d} expected>={d}", .{
                    value.size,
                    @sizeOf(Size),
                });
                return;
            }

            const renderer = value.toRenderer() orelse {
                log.warn("setopt invalid size values (cell width and height must be non-zero)", .{});
                return;
            };

            if (comptime build_options.lib_vt_rust) {
                var last_cell_present = wrapper.last_cell != null;
                rust.ghostty_rust_mouse_encoder_setopt_size(
                    value.screen_width,
                    value.screen_height,
                    value.cell_width,
                    value.cell_height,
                    value.padding_top,
                    value.padding_bottom,
                    value.padding_right,
                    value.padding_left,
                    &wrapper.opts.size.screen.width,
                    &wrapper.opts.size.screen.height,
                    &wrapper.opts.size.cell.width,
                    &wrapper.opts.size.cell.height,
                    &wrapper.opts.size.padding.top,
                    &wrapper.opts.size.padding.bottom,
                    &wrapper.opts.size.padding.right,
                    &wrapper.opts.size.padding.left,
                    &last_cell_present,
                );
                if (!last_cell_present) wrapper.last_cell = null;
            } else {
                wrapper.opts.size = renderer;
                wrapper.last_cell = null;
            }
        },

        .any_button_pressed => setoptBool(option, value.*, wrapper),

        .track_last_cell => {
            if (comptime build_options.lib_vt_rust) {
                setoptBool(option, value.*, wrapper);
            } else {
                wrapper.track_last_cell = value.*;
                if (!value.*) wrapper.last_cell = null;
            }
        },
    }
}

fn setoptBool(
    option: Option,
    value: bool,
    wrapper: *MouseEncoderWrapper,
) void {
    if (comptime build_options.lib_vt_rust) {
        var last_cell_present = wrapper.last_cell != null;
        rust.ghostty_rust_mouse_encoder_setopt_bool(
            @intFromEnum(option),
            value,
            &wrapper.opts.any_button_pressed,
            &wrapper.track_last_cell,
            &last_cell_present,
        );
        if (!last_cell_present) wrapper.last_cell = null;
    } else {
        switch (option) {
            .any_button_pressed => wrapper.opts.any_button_pressed = value,
            .track_last_cell => {
                wrapper.track_last_cell = value;
                if (!value) wrapper.last_cell = null;
            },
            else => unreachable,
        }
    }
}

pub fn setopt_from_terminal(
    encoder_: Encoder,
    terminal_: Terminal,
) callconv(lib.calling_conv) void {
    const wrapper = encoder_ orelse return;
    const t: *ZigTerminal = @import("terminal.zig").terminalZig(terminal_) orelse return;
    if (comptime build_options.lib_vt_rust) {
        var last_cell_present = wrapper.last_cell != null;
        rust.ghostty_rust_mouse_encoder_from_terminal(
            @intFromEnum(t.flags.mouse_event),
            @intFromEnum(t.flags.mouse_format),
            &wrapper.opts.event,
            &wrapper.opts.format,
            &last_cell_present,
        );
        if (!last_cell_present) wrapper.last_cell = null;
    } else {
        wrapper.opts.event = t.flags.mouse_event;
        wrapper.opts.format = t.flags.mouse_format;
        wrapper.last_cell = null;
    }
}

pub fn reset(encoder_: Encoder) callconv(lib.calling_conv) void {
    const wrapper = encoder_ orelse return;
    if (comptime build_options.lib_vt_rust) {
        var last_cell_present = wrapper.last_cell != null;
        rust.ghostty_rust_mouse_encoder_reset(&last_cell_present);
        if (!last_cell_present) wrapper.last_cell = null;
    } else {
        wrapper.last_cell = null;
    }
}

pub fn encode(
    encoder_: Encoder,
    event_: Event,
    out_: ?[*]u8,
    out_len: usize,
    out_written: *usize,
) callconv(lib.calling_conv) Result {
    const wrapper = encoder_ orelse return .invalid_value;
    const event = event_ orelse return .invalid_value;

    if (comptime build_options.lib_vt_rust) {
        const last_cell = wrapper.last_cell;
        const button_present = event.event.button != null;
        const button: mouse_event.Button = event.event.button orelse .unknown;

        var next_last_cell_present = last_cell != null;
        var next_last_cell_x: u16 = if (last_cell) |cell| cell.x else 0;
        var next_last_cell_y: u32 = if (last_cell) |cell| cell.y else 0;

        const result_int = rust.ghostty_rust_mouse_encode(
            @intFromEnum(event.event.action),
            button_present,
            @intFromEnum(button),
            @bitCast(event.event.mods),
            event.event.pos,
            @intFromEnum(wrapper.opts.event),
            @intFromEnum(wrapper.opts.format),
            wrapper.opts.size.screen.width,
            wrapper.opts.size.screen.height,
            wrapper.opts.size.cell.width,
            wrapper.opts.size.cell.height,
            wrapper.opts.size.padding.top,
            wrapper.opts.size.padding.bottom,
            wrapper.opts.size.padding.right,
            wrapper.opts.size.padding.left,
            wrapper.opts.any_button_pressed,
            wrapper.track_last_cell,
            last_cell != null,
            if (last_cell) |cell| cell.x else 0,
            if (last_cell) |cell| cell.y else 0,
            out_,
            out_len,
            out_written,
            &next_last_cell_present,
            &next_last_cell_x,
            &next_last_cell_y,
        );

        const result: Result = @enumFromInt(result_int);
        if (result == .success and wrapper.track_last_cell) {
            wrapper.last_cell = if (next_last_cell_present) .{
                .x = next_last_cell_x,
                .y = next_last_cell_y,
            } else null;
        }

        return result;
    }

    const prev_last_cell = wrapper.last_cell;

    var opts = wrapper.opts;
    opts.last_cell = if (wrapper.track_last_cell) &wrapper.last_cell else null;

    var writer: std.Io.Writer = .fixed(if (out_) |out| out[0..out_len] else &.{});
    input_mouse_encode.encode(
        &writer,
        event.event,
        opts,
    ) catch |err| switch (err) {
        error.WriteFailed => {
            // Failed writes should not mutate motion dedupe state because no
            // complete sequence was produced.
            wrapper.last_cell = prev_last_cell;

            // Use a discarding writer to count how much space we would have needed.
            var count_last_cell = prev_last_cell;
            var count_opts = wrapper.opts;
            count_opts.last_cell = if (wrapper.track_last_cell) &count_last_cell else null;

            var discarding: std.Io.Writer.Discarding = .init(&.{});
            input_mouse_encode.encode(
                &discarding.writer,
                event.event,
                count_opts,
            ) catch unreachable;

            // Discarding always uses a u64. If we're on 32-bit systems
            // we cast down. We should make this safer in the future.
            out_written.* = @intCast(discarding.count);
            return .out_of_space;
        },
    };

    out_written.* = writer.end;
    return .success;
}

fn defaultSize() renderer_size.Size {
    return .{
        .screen = .{ .width = 1, .height = 1 },
        .cell = .{ .width = 1, .height = 1 },
        .padding = .{},
    };
}

fn testSize() Size {
    return .{
        .size = @sizeOf(Size),
        .screen_width = 1_000,
        .screen_height = 1_000,
        .cell_width = 1,
        .cell_height = 1,
        .padding_top = 0,
        .padding_bottom = 0,
        .padding_right = 0,
        .padding_left = 0,
    };
}

test "alloc" {
    var e: Encoder = undefined;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &e,
    ));
    free(e);
}

test "setopt" {
    var e: Encoder = undefined;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &e,
    ));
    defer free(e);

    const event_mode: TrackingMode = .any;
    setopt(e, .event, &event_mode);
    try testing.expectEqual(TrackingMode.any, e.?.opts.event);

    const format_mode: Format = .sgr;
    setopt(e, .format, &format_mode);
    try testing.expectEqual(Format.sgr, e.?.opts.format);

    const size = testSize();
    setopt(e, .size, &size);
    try testing.expectEqual(size.screen_width, e.?.opts.size.screen.width);
    try testing.expectEqual(size.screen_height, e.?.opts.size.screen.height);

    const any_button_pressed = true;
    setopt(e, .any_button_pressed, &any_button_pressed);
    try testing.expect(e.?.opts.any_button_pressed);

    const track_last_cell = true;
    setopt(e, .track_last_cell, &track_last_cell);
    try testing.expect(e.?.track_last_cell);
}

test "setopt clears last cell state" {
    var e: Encoder = undefined;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &e,
    ));
    defer free(e);

    e.?.last_cell = .{ .x = 1, .y = 2 };
    const event_mode: TrackingMode = .any;
    setopt(e, .event, &event_mode);
    try testing.expect(e.?.last_cell == null);

    e.?.last_cell = .{ .x = 1, .y = 2 };
    const format_mode: Format = .sgr;
    setopt(e, .format, &format_mode);
    try testing.expect(e.?.last_cell == null);

    e.?.last_cell = .{ .x = 1, .y = 2 };
    const size = testSize();
    setopt(e, .size, &size);
    try testing.expect(e.?.last_cell == null);

    e.?.last_cell = .{ .x = 1, .y = 2 };
    const track_last_cell = false;
    setopt(e, .track_last_cell, &track_last_cell);
    try testing.expect(e.?.last_cell == null);
}

test "setopt_from_terminal" {
    const terminal_c = @import("terminal.zig");

    var e: Encoder = undefined;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &e,
    ));
    defer free(e);

    var t: Terminal = undefined;
    try testing.expectEqual(Result.success, terminal_c.new(
        &lib.alloc.test_allocator,
        &t,
        .{ .cols = 80, .rows = 24, .max_scrollback = 0 },
    ));
    defer terminal_c.free(t);

    const event_mode: TrackingMode = .any;
    setopt(e, .event, &event_mode);
    const format_mode: Format = .sgr;
    setopt(e, .format, &format_mode);

    e.?.last_cell = .{ .x = 1, .y = 2 };
    setopt_from_terminal(e, t);
    try testing.expectEqual(TrackingMode.none, e.?.opts.event);
    try testing.expectEqual(Format.x10, e.?.opts.format);
    try testing.expect(e.?.last_cell == null);
}

test "setopt_from_terminal null" {
    setopt_from_terminal(null, null);

    const terminal_c = @import("terminal.zig");
    var t: Terminal = undefined;
    try testing.expectEqual(Result.success, terminal_c.new(
        &lib.alloc.test_allocator,
        &t,
        .{ .cols = 80, .rows = 24, .max_scrollback = 0 },
    ));
    defer terminal_c.free(t);
    setopt_from_terminal(null, t);

    var e: Encoder = undefined;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &e,
    ));
    defer free(e);
    setopt_from_terminal(e, null);
}

test "reset clears last cell" {
    var e: Encoder = undefined;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &e,
    ));
    defer free(e);

    e.?.last_cell = .{ .x = 5, .y = 6 };
    reset(e);
    try testing.expect(e.?.last_cell == null);
    reset(null);
}

test "encode: sgr press left" {
    var encoder: Encoder = undefined;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &encoder,
    ));
    defer free(encoder);

    const event_mode: TrackingMode = .any;
    setopt(encoder, .event, &event_mode);
    const format_mode: Format = .sgr;
    setopt(encoder, .format, &format_mode);
    const size = testSize();
    setopt(encoder, .size, &size);

    var event: Event = undefined;
    try testing.expectEqual(Result.success, mouse_event.new(
        &lib.alloc.test_allocator,
        &event,
    ));
    defer mouse_event.free(event);

    mouse_event.set_action(event, .press);
    mouse_event.set_button(event, .left);
    mouse_event.set_position(event, .{ .x = 0, .y = 0 });

    var required: usize = 0;
    try testing.expectEqual(Result.out_of_space, encode(
        encoder,
        event,
        null,
        0,
        &required,
    ));
    try testing.expectEqual(@as(usize, 9), required);

    var buf: [32]u8 = undefined;
    var written: usize = 0;
    try testing.expectEqual(Result.success, encode(
        encoder,
        event,
        &buf,
        buf.len,
        &written,
    ));
    try testing.expectEqual(required, written);
    try testing.expectEqualStrings("\x1b[<0;1;1M", buf[0..written]);
}

test "encode: motion dedupe and reset" {
    var encoder: Encoder = undefined;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &encoder,
    ));
    defer free(encoder);

    const event_mode: TrackingMode = .any;
    setopt(encoder, .event, &event_mode);
    const format_mode: Format = .sgr;
    setopt(encoder, .format, &format_mode);
    const size = testSize();
    setopt(encoder, .size, &size);
    const track_last_cell = true;
    setopt(encoder, .track_last_cell, &track_last_cell);

    var event: Event = undefined;
    try testing.expectEqual(Result.success, mouse_event.new(
        &lib.alloc.test_allocator,
        &event,
    ));
    defer mouse_event.free(event);

    mouse_event.set_action(event, .motion);
    mouse_event.set_button(event, .left);
    mouse_event.set_position(event, .{ .x = 5, .y = 6 });

    {
        var buf: [32]u8 = undefined;
        var written: usize = 0;
        try testing.expectEqual(Result.success, encode(
            encoder,
            event,
            &buf,
            buf.len,
            &written,
        ));
        try testing.expect(written > 0);
    }

    {
        var buf: [32]u8 = undefined;
        var written: usize = 0;
        try testing.expectEqual(Result.success, encode(
            encoder,
            event,
            &buf,
            buf.len,
            &written,
        ));
        try testing.expectEqual(@as(usize, 0), written);
    }

    reset(encoder);

    {
        var buf: [32]u8 = undefined;
        var written: usize = 0;
        try testing.expectEqual(Result.success, encode(
            encoder,
            event,
            &buf,
            buf.len,
            &written,
        ));
        try testing.expect(written > 0);
    }
}

test "encode: querying required size doesn't update dedupe state" {
    var encoder: Encoder = undefined;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &encoder,
    ));
    defer free(encoder);

    const event_mode: TrackingMode = .any;
    setopt(encoder, .event, &event_mode);
    const format_mode: Format = .sgr;
    setopt(encoder, .format, &format_mode);
    const size = testSize();
    setopt(encoder, .size, &size);
    const track_last_cell = true;
    setopt(encoder, .track_last_cell, &track_last_cell);

    var event: Event = undefined;
    try testing.expectEqual(Result.success, mouse_event.new(
        &lib.alloc.test_allocator,
        &event,
    ));
    defer mouse_event.free(event);

    mouse_event.set_action(event, .motion);
    mouse_event.set_button(event, .left);
    mouse_event.set_position(event, .{ .x = 5, .y = 6 });

    var required: usize = 0;
    try testing.expectEqual(Result.out_of_space, encode(
        encoder,
        event,
        null,
        0,
        &required,
    ));
    try testing.expect(required > 0);

    var buf: [32]u8 = undefined;
    var written: usize = 0;
    try testing.expectEqual(Result.success, encode(
        encoder,
        event,
        &buf,
        buf.len,
        &written,
    ));
    try testing.expect(written > 0);
}
