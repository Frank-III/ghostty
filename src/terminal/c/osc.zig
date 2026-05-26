const std = @import("std");
const build_options = @import("terminal_options");
const lib = @import("../lib.zig");
const CAllocator = lib.alloc.Allocator;
const osc = @import("../osc.zig");
const Result = @import("result.zig").Result;

const log = std.log.scoped(.osc);

const rust = if (build_options.lib_vt_rust) struct {
    extern fn ghostty_rust_osc_command_type(
        has_command: bool,
        kind: c_int,
    ) callconv(.c) c_int;

    extern fn ghostty_rust_osc_command_data_string(
        data: c_int,
        has_value: bool,
        value: ?[*:0]const u8,
        out: ?*anyopaque,
    ) callconv(.c) bool;
} else struct {};

/// C: GhosttyOscParser
pub const Parser = ?*osc.Parser;

/// C: GhosttyOscCommand
pub const Command = ?*osc.Command;

pub fn new(
    alloc_: ?*const CAllocator,
    result: *Parser,
) callconv(lib.calling_conv) Result {
    const alloc = lib.alloc.default(alloc_);
    const ptr = alloc.create(osc.Parser) catch
        return .out_of_memory;
    ptr.* = .init(alloc);
    result.* = ptr;
    return .success;
}

pub fn free(parser_: Parser) callconv(lib.calling_conv) void {
    // C-built parsers always have an associated allocator.
    const parser = parser_ orelse return;
    const alloc = parser.alloc.?;
    parser.deinit();
    alloc.destroy(parser);
}

pub fn reset(parser_: Parser) callconv(lib.calling_conv) void {
    parser_.?.reset();
}

pub fn next(parser_: Parser, byte: u8) callconv(lib.calling_conv) void {
    parser_.?.next(byte);
}

pub fn end(parser_: Parser, terminator: u8) callconv(lib.calling_conv) Command {
    return parser_.?.end(terminator);
}

pub fn commandType(command_: Command) callconv(lib.calling_conv) osc.Command.Key {
    if (comptime build_options.lib_vt_rust) {
        return @enumFromInt(rust.ghostty_rust_osc_command_type(
            command_ != null,
            if (command_) |command| @intFromEnum(command.*) else @intFromEnum(osc.Command.Key.invalid),
        ));
    }

    const command = command_ orelse return .invalid;
    return command.*;
}

/// C: GhosttyOscCommandData
pub const CommandData = enum(c_int) {
    invalid = 0,
    change_window_title_str = 1,

    /// Output type expected for querying the data of the given kind.
    pub fn OutType(comptime self: CommandData) type {
        return switch (self) {
            .invalid => void,
            .change_window_title_str => [*:0]const u8,
        };
    }
};

pub fn commandData(
    command_: Command,
    data: CommandData,
    out: ?*anyopaque,
) callconv(lib.calling_conv) bool {
    if (comptime std.debug.runtime_safety) {
        _ = std.meta.intToEnum(CommandData, @intFromEnum(data)) catch {
            log.warn("commandData invalid data value={d}", .{@intFromEnum(data)});
            return false;
        };
    }

    if (data != .invalid and out == null) return false;

    if (comptime build_options.lib_vt_rust) {
        switch (data) {
            .invalid => return false,
            .change_window_title_str => {
                const command = command_ orelse return false;
                const title: ?[:0]const u8 = switch (command.*) {
                    .change_window_title => |v| v,
                    else => null,
                };
                return rust.ghostty_rust_osc_command_data_string(
                    @intFromEnum(data),
                    title != null,
                    if (title) |v| v.ptr else null,
                    out,
                );
            },
        }
    }

    return switch (data) {
        .invalid => false,
        inline else => |comptime_data| commandDataTyped(
            command_,
            comptime_data,
            @ptrCast(@alignCast(out)),
        ),
    };
}

fn commandDataTyped(
    command_: Command,
    comptime data: CommandData,
    out: *data.OutType(),
) bool {
    const command = command_ orelse return false;
    switch (data) {
        .invalid => return false,
        .change_window_title_str => switch (command.*) {
            .change_window_title => |v| out.* = v.ptr,
            else => return false,
        },
    }

    return true;
}

test "alloc" {
    const testing = std.testing;
    var p: Parser = undefined;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &p,
    ));
    free(p);
}

test "command type null" {
    const testing = std.testing;
    try testing.expectEqual(.invalid, commandType(null));
}

test "change window title" {
    const testing = std.testing;
    var p: Parser = undefined;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &p,
    ));
    defer free(p);

    // Parse it
    next(p, '0');
    next(p, ';');
    next(p, 'a');
    const cmd = end(p, 0);
    try testing.expectEqual(.change_window_title, commandType(cmd));

    // Extract the title
    var title: [*:0]const u8 = undefined;
    try testing.expect(commandData(cmd, .change_window_title_str, @ptrCast(&title)));
    try testing.expectEqualStrings("a", std.mem.span(title));
}

test "command data invalid values" {
    const testing = std.testing;
    var title: [*:0]const u8 = undefined;
    try testing.expect(!commandData(null, .change_window_title_str, @ptrCast(&title)));

    var p: Parser = undefined;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &p,
    ));
    defer free(p);

    next(p, '0');
    next(p, ';');
    next(p, 'a');
    const cmd = end(p, 0);
    try testing.expect(!commandData(cmd, .change_window_title_str, null));
}
