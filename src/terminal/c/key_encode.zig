const std = @import("std");
const Allocator = std.mem.Allocator;
const build_options = @import("terminal_options");
const lib = @import("../lib.zig");
const CAllocator = lib.alloc.Allocator;
const key_encode = @import("../../input/key_encode.zig");
const key_event = @import("key_event.zig");
const KittyFlags = @import("../../terminal/kitty/key.zig").Flags;
const OptionAsAlt = @import("../../input/config.zig").OptionAsAlt;
const Result = @import("result.zig").Result;
const KeyEvent = @import("key_event.zig").Event;
const Terminal = @import("terminal.zig").Terminal;
const terminal_c = @import("terminal.zig");
const ZigTerminal = @import("../Terminal.zig");

const log = std.log.scoped(.key_encode);

const rust_owned = if (build_options.terminal_rust_owned) struct {
    extern fn ghostty_rust_terminal_owned_set_modify_other_keys_2(
        handle: ?*anyopaque,
        value: bool,
    ) callconv(.c) void;

    extern fn ghostty_rust_terminal_owned_key_encoder_from_terminal(
        handle: ?*anyopaque,
        out_alt_esc_prefix: *bool,
        out_cursor_key_application: *bool,
        out_keypad_key_application: *bool,
        out_backarrow_key_mode: *bool,
        out_ignore_keypad_with_numlock: *bool,
        out_modify_other_keys_state_2: *bool,
        out_macos_option_as_alt: *c_int,
        out_kitty_flags: *u8,
    ) callconv(.c) void;
} else struct {};

const rust = if (build_options.lib_vt_rust) struct {
    extern fn ghostty_rust_key_encoder_setopt_bool(
        option: c_int,
        value: bool,
        out: *bool,
    ) callconv(.c) void;

    extern fn ghostty_rust_key_encoder_setopt_option_as_alt(
        option: c_int,
        value: c_int,
        out: *OptionAsAlt,
    ) callconv(.c) void;

    extern fn ghostty_rust_key_encoder_from_terminal(
        alt_esc_prefix: bool,
        cursor_key_application: bool,
        keypad_key_application: bool,
        backarrow_key_mode: bool,
        ignore_keypad_with_numlock: bool,
        modify_other_keys_state_2: bool,
        out_alt_esc_prefix: *bool,
        out_cursor_key_application: *bool,
        out_keypad_key_application: *bool,
        out_backarrow_key_mode: *bool,
        out_ignore_keypad_with_numlock: *bool,
        out_modify_other_keys_state_2: *bool,
        out_macos_option_as_alt: *OptionAsAlt,
    ) callconv(.c) void;
} else struct {};

/// Wrapper around key encoding options that tracks the allocator for C API usage.
const KeyEncoderWrapper = struct {
    opts: key_encode.Options,
    alloc: Allocator,
};

/// C: GhosttyKeyEncoder
pub const Encoder = ?*KeyEncoderWrapper;

pub fn new(
    alloc_: ?*const CAllocator,
    result: *Encoder,
) callconv(lib.calling_conv) Result {
    const alloc = lib.alloc.default(alloc_);
    const ptr = alloc.create(KeyEncoderWrapper) catch
        return .out_of_memory;
    ptr.* = .{
        .opts = .{},
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

/// C: GhosttyKeyEncoderOption
pub const Option = enum(c_int) {
    cursor_key_application = 0,
    keypad_key_application = 1,
    ignore_keypad_with_numlock = 2,
    alt_esc_prefix = 3,
    modify_other_keys_state_2 = 4,
    kitty_flags = 5,
    macos_option_as_alt = 6,
    /// DEC Backarrow Key Mode (DECBKM)
    /// See https://vt100.net/dec/ek-vt3xx-tp-002.pdf page 170
    /// If `false` (the default), `backspace` emits 0x7f
    /// If `true`, `backspace` emits 0x08
    backarrow_key_mode = 7,

    /// Input type expected for setting the option.
    pub fn InType(comptime self: Option) type {
        return switch (self) {
            .cursor_key_application,
            .keypad_key_application,
            .ignore_keypad_with_numlock,
            .alt_esc_prefix,
            .modify_other_keys_state_2,
            .backarrow_key_mode,
            => bool,
            .kitty_flags => u8,
            .macos_option_as_alt => OptionAsAlt,
        };
    }
};

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
    const opts = &encoder_.?.opts;
    switch (option) {
        .cursor_key_application => setoptBool(option, value.*, &opts.cursor_key_application),
        .keypad_key_application => setoptBool(option, value.*, &opts.keypad_key_application),
        .ignore_keypad_with_numlock => setoptBool(option, value.*, &opts.ignore_keypad_with_numlock),
        .alt_esc_prefix => setoptBool(option, value.*, &opts.alt_esc_prefix),
        .modify_other_keys_state_2 => setoptBool(option, value.*, &opts.modify_other_keys_state_2),
        .kitty_flags => opts.kitty_flags = flags: {
            const bits: u5 = @truncate(value.*);
            break :flags @bitCast(bits);
        },
        .macos_option_as_alt => {
            if (comptime std.debug.runtime_safety) {
                _ = std.meta.intToEnum(OptionAsAlt, @intFromEnum(value.*)) catch {
                    log.warn("setopt invalid OptionAsAlt value={d}", .{@intFromEnum(value.*)});
                    return;
                };
            }
            if (comptime build_options.lib_vt_rust) {
                rust.ghostty_rust_key_encoder_setopt_option_as_alt(
                    @intFromEnum(option),
                    @intFromEnum(value.*),
                    &opts.macos_option_as_alt,
                );
            } else {
                opts.macos_option_as_alt = value.*;
            }
        },
        .backarrow_key_mode => setoptBool(option, value.*, &opts.backarrow_key_mode),
    }
}

fn setoptBool(
    option: Option,
    value: bool,
    out: *bool,
) void {
    if (comptime build_options.lib_vt_rust) {
        rust.ghostty_rust_key_encoder_setopt_bool(
            @intFromEnum(option),
            value,
            out,
        );
    } else {
        out.* = value;
    }
}

pub fn setopt_from_terminal(
    encoder_: Encoder,
    terminal_: Terminal,
) callconv(lib.calling_conv) void {
    const wrapper = encoder_ orelse return;
    const term_wrapper = terminal_ orelse return;

    if (comptime build_options.terminal_rust_owned) {
        if (terminal_c.rustOwnedHandle(term_wrapper)) |handle| {
            const opts = &wrapper.opts;
            var kitty_raw: u8 = undefined;
            rust_owned.ghostty_rust_terminal_owned_key_encoder_from_terminal(
                handle,
                &opts.alt_esc_prefix,
                &opts.cursor_key_application,
                &opts.keypad_key_application,
                &opts.backarrow_key_mode,
                &opts.ignore_keypad_with_numlock,
                &opts.modify_other_keys_state_2,
                @ptrCast(&opts.macos_option_as_alt),
                &kitty_raw,
            );
            opts.kitty_flags = @bitCast(@as(u5, @truncate(kitty_raw)));
            return;
        }
    }

    const t: *ZigTerminal = terminal_c.terminalZig(terminal_) orelse return;
    if (comptime build_options.lib_vt_rust) {
        const opts = &wrapper.opts;
        rust.ghostty_rust_key_encoder_from_terminal(
            t.modes.get(.alt_esc_prefix),
            t.modes.get(.cursor_keys),
            t.modes.get(.keypad_keys),
            t.modes.get(.backarrow_key_mode),
            t.modes.get(.ignore_keypad_with_numlock),
            t.flags.modify_other_keys_2,
            &opts.alt_esc_prefix,
            &opts.cursor_key_application,
            &opts.keypad_key_application,
            &opts.backarrow_key_mode,
            &opts.ignore_keypad_with_numlock,
            &opts.modify_other_keys_state_2,
            &opts.macos_option_as_alt,
        );
        opts.kitty_flags = t.screens.active.kitty_keyboard.current();
    } else {
        wrapper.opts = .fromTerminal(t);
    }
}

pub fn encode(
    encoder_: Encoder,
    event_: KeyEvent,
    out_: ?[*]u8,
    out_len: usize,
    out_written: *usize,
) callconv(lib.calling_conv) Result {
    // Attempt to write to this buffer
    var writer: std.Io.Writer = .fixed(if (out_) |out| out[0..out_len] else &.{});
    key_encode.encode(
        &writer,
        event_.?.event,
        encoder_.?.opts,
    ) catch |err| switch (err) {
        error.WriteFailed => {
            // If we don't have space, use a discarding writer to count
            // how much space we would have needed.
            var discarding: std.Io.Writer.Discarding = .init(&.{});
            key_encode.encode(
                &discarding.writer,
                event_.?.event,
                encoder_.?.opts,
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

test "alloc" {
    const testing = std.testing;
    var e: Encoder = undefined;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &e,
    ));
    free(e);
}

test "setopt bool" {
    const testing = std.testing;
    var e: Encoder = undefined;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &e,
    ));
    defer free(e);

    // Test setting bool options
    const val_true: bool = true;
    setopt(e, .cursor_key_application, &val_true);
    try testing.expect(e.?.opts.cursor_key_application);

    const val_false: bool = false;
    setopt(e, .cursor_key_application, &val_false);
    try testing.expect(!e.?.opts.cursor_key_application);

    setopt(e, .keypad_key_application, &val_true);
    try testing.expect(e.?.opts.keypad_key_application);
}

test "setopt all bool options" {
    const testing = std.testing;
    var e: Encoder = undefined;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &e,
    ));
    defer free(e);

    const val_true: bool = true;
    setopt(e, .cursor_key_application, &val_true);
    setopt(e, .keypad_key_application, &val_true);
    setopt(e, .ignore_keypad_with_numlock, &val_true);
    setopt(e, .alt_esc_prefix, &val_true);
    setopt(e, .modify_other_keys_state_2, &val_true);
    setopt(e, .backarrow_key_mode, &val_true);

    try testing.expect(e.?.opts.cursor_key_application);
    try testing.expect(e.?.opts.keypad_key_application);
    try testing.expect(e.?.opts.ignore_keypad_with_numlock);
    try testing.expect(e.?.opts.alt_esc_prefix);
    try testing.expect(e.?.opts.modify_other_keys_state_2);
    try testing.expect(e.?.opts.backarrow_key_mode);
}

test "setopt kitty flags" {
    const testing = std.testing;
    var e: Encoder = undefined;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &e,
    ));
    defer free(e);

    // Test setting kitty flags
    const flags: KittyFlags = .{
        .disambiguate = true,
        .report_events = true,
    };
    const flags_int: u8 = @intCast(flags.int());
    setopt(e, .kitty_flags, &flags_int);
    try testing.expect(e.?.opts.kitty_flags.disambiguate);
    try testing.expect(e.?.opts.kitty_flags.report_events);
    try testing.expect(!e.?.opts.kitty_flags.report_alternates);
}

test "setopt macos option as alt" {
    const testing = std.testing;
    var e: Encoder = undefined;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &e,
    ));
    defer free(e);

    // Test setting option as alt
    const opt_left: OptionAsAlt = .left;
    setopt(e, .macos_option_as_alt, &opt_left);
    try testing.expectEqual(OptionAsAlt.left, e.?.opts.macos_option_as_alt);

    const opt_true: OptionAsAlt = .true;
    setopt(e, .macos_option_as_alt, &opt_true);
    try testing.expectEqual(OptionAsAlt.true, e.?.opts.macos_option_as_alt);
}

test "setopt_from_terminal" {
    const testing = std.testing;
    const modes = @import("../modes.zig");

    // Create encoder
    var e: Encoder = undefined;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &e,
    ));
    defer free(e);

    // Create terminal
    var t: Terminal = undefined;
    try testing.expectEqual(Result.success, terminal_c.new(
        &lib.alloc.test_allocator,
        &t,
        .{ .cols = 80, .rows = 24, .max_scrollback = 0 },
    ));
    defer terminal_c.free(t);

    const cursor_keys: modes.ModeTag.Backing = @bitCast(modes.ModeTag{ .value = 1, .ansi = false });
    const keypad_keys: modes.ModeTag.Backing = @bitCast(modes.ModeTag{ .value = 66, .ansi = false });
    const backarrow_key_mode: modes.ModeTag.Backing = @bitCast(modes.ModeTag{ .value = 67, .ansi = false });
    const ignore_keypad_with_numlock: modes.ModeTag.Backing = @bitCast(modes.ModeTag{ .value = 1035, .ansi = false });
    try testing.expectEqual(Result.success, terminal_c.mode_set(t, cursor_keys, true));
    try testing.expectEqual(Result.success, terminal_c.mode_set(t, keypad_keys, true));
    try testing.expectEqual(Result.success, terminal_c.mode_set(t, backarrow_key_mode, true));
    try testing.expectEqual(Result.success, terminal_c.mode_set(t, ignore_keypad_with_numlock, false));
    if (comptime build_options.terminal_rust_owned) {
        const handle = terminal_c.rustOwnedHandle(t.?) orelse return;
        rust_owned.ghostty_rust_terminal_owned_set_modify_other_keys_2(handle, true);
    } else {
        terminal_c.terminalZig(t).?.flags.modify_other_keys_2 = true;
    }

    const opt_true: OptionAsAlt = .true;
    setopt(e, .macos_option_as_alt, &opt_true);
    try testing.expectEqual(OptionAsAlt.true, e.?.opts.macos_option_as_alt);

    // Apply terminal state to encoder
    setopt_from_terminal(e, t);

    // Options should reflect terminal state, while config-only values reset.
    try testing.expect(e.?.opts.cursor_key_application);
    try testing.expect(e.?.opts.keypad_key_application);
    try testing.expect(e.?.opts.backarrow_key_mode);
    try testing.expect(!e.?.opts.ignore_keypad_with_numlock);
    try testing.expect(e.?.opts.alt_esc_prefix);
    try testing.expect(e.?.opts.modify_other_keys_state_2);
    try testing.expectEqual(KittyFlags.disabled, e.?.opts.kitty_flags);
    try testing.expectEqual(OptionAsAlt.false, e.?.opts.macos_option_as_alt);
}

test "setopt_from_terminal null" {
    // Both null should be no-ops
    setopt_from_terminal(null, null);

    const testing = std.testing;

    // Encoder null with valid terminal
    var t: Terminal = undefined;
    try testing.expectEqual(Result.success, terminal_c.new(
        &lib.alloc.test_allocator,
        &t,
        .{ .cols = 80, .rows = 24, .max_scrollback = 0 },
    ));
    defer terminal_c.free(t);
    setopt_from_terminal(null, t);

    // Valid encoder with null terminal
    var e: Encoder = undefined;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &e,
    ));
    defer free(e);
    setopt_from_terminal(e, null);
}

test "encode: kitty ctrl release with ctrl mod set" {
    const testing = std.testing;

    // Create encoder
    var encoder: Encoder = undefined;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &encoder,
    ));
    defer free(encoder);

    // Set kitty flags with all features enabled
    {
        const flags: KittyFlags = .{
            .disambiguate = true,
            .report_events = true,
            .report_alternates = true,
            .report_all = true,
            .report_associated = true,
        };
        const flags_int: u8 = @intCast(flags.int());
        setopt(encoder, .kitty_flags, &flags_int);
    }

    // Create key event
    var event: key_event.Event = undefined;
    try testing.expectEqual(Result.success, key_event.new(
        &lib.alloc.test_allocator,
        &event,
    ));
    defer key_event.free(event);

    // Set event properties: release action, ctrl key, ctrl modifier
    key_event.set_action(event, .release);
    key_event.set_key(event, .control_left);
    key_event.set_mods(event, .{ .ctrl = true });

    // Encode null should give us the length required
    var required: usize = 0;
    try testing.expectEqual(Result.out_of_space, encode(
        encoder,
        event,
        null,
        0,
        &required,
    ));

    // Encode the key event
    var buf: [128]u8 = undefined;
    var written: usize = 0;
    try testing.expectEqual(Result.success, encode(
        encoder,
        event,
        &buf,
        buf.len,
        &written,
    ));
    try testing.expectEqual(required, written);

    // Expected: ESC[57442;5:3u (ctrl key code with mods and release event)
    const actual = buf[0..written];
    try testing.expectEqualStrings("\x1b[57442;5:3u", actual);
}
