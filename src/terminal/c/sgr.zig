const std = @import("std");
const testing = std.testing;
const Allocator = std.mem.Allocator;
const lib = @import("../lib.zig");
const build_options = @import("terminal_options");
const CAllocator = lib.alloc.Allocator;
const sgr = @import("../sgr.zig");
const Result = @import("result.zig").Result;

const log = std.log.scoped(.sgr);

/// Wrapper around parser that tracks the allocator for C API usage.
const ParserWrapper = struct {
    parser: sgr.Parser,
    alloc: Allocator,
};

/// C: GhosttySgrParser
pub const Parser = ?*ParserWrapper;

const rust = if (build_options.lib_vt_rust) struct {
    extern fn ghostty_rust_sgr_parser_reset(
        idx: *usize,
    ) callconv(.c) void;

    extern fn ghostty_rust_sgr_params_sep_mask(
        seps: [*]const u8,
        len: usize,
    ) callconv(.c) u32;

    extern fn ghostty_rust_sgr_unknown_full(
        unknown: sgr.Attribute.Unknown.C,
        ptr: ?*[*]const u16,
    ) callconv(.c) usize;

    extern fn ghostty_rust_sgr_unknown_partial(
        unknown: sgr.Attribute.Unknown.C,
        ptr: ?*[*]const u16,
    ) callconv(.c) usize;

    extern fn ghostty_rust_sgr_attribute_tag(
        attr: sgr.Attribute.C,
    ) callconv(.c) sgr.Attribute.Tag;

    extern fn ghostty_rust_sgr_attribute_value(
        attr: *sgr.Attribute.C,
    ) callconv(.c) *sgr.Attribute.CValue;
} else struct {};

pub fn new(
    alloc_: ?*const CAllocator,
    result: *Parser,
) callconv(lib.calling_conv) Result {
    const alloc = lib.alloc.default(alloc_);
    const ptr = alloc.create(ParserWrapper) catch
        return .out_of_memory;
    ptr.* = .{
        .parser = .empty,
        .alloc = alloc,
    };
    result.* = ptr;
    return .success;
}

pub fn free(parser_: Parser) callconv(lib.calling_conv) void {
    const wrapper = parser_ orelse return;
    const alloc = wrapper.alloc;
    const parser: *sgr.Parser = &wrapper.parser;
    if (parser.params.len > 0) alloc.free(parser.params);
    alloc.destroy(wrapper);
}

pub fn reset(parser_: Parser) callconv(lib.calling_conv) void {
    const wrapper = parser_ orelse return;
    const parser: *sgr.Parser = &wrapper.parser;
    resetIdx(parser);
}

pub fn setParams(
    parser_: Parser,
    params: [*]const u16,
    seps_: ?[*]const u8,
    len: usize,
) callconv(lib.calling_conv) Result {
    const wrapper = parser_ orelse return .invalid_value;
    const alloc = wrapper.alloc;
    const parser: *sgr.Parser = &wrapper.parser;

    // Copy our new parameters
    const params_slice = alloc.dupe(u16, params[0..len]) catch
        return .out_of_memory;
    if (parser.params.len > 0) alloc.free(parser.params);
    parser.params = params_slice;

    // If we have separators, set that state too.
    parser.params_sep = .initEmpty();
    if (seps_) |seps| {
        if (len > @TypeOf(parser.params_sep).bit_length) {
            log.warn("ghostty_sgr_set_params: separators length {} exceeds max supported length {}", .{
                len,
                @TypeOf(parser.params_sep).bit_length,
            });
            return .invalid_value;
        }

        setParamsSep(parser, seps, len);
    }

    // Reset our parsing state
    resetIdx(parser);

    return .success;
}

fn setParamsSep(parser: *sgr.Parser, seps: [*]const u8, len: usize) void {
    if (comptime build_options.lib_vt_rust) {
        const mask = rust.ghostty_rust_sgr_params_sep_mask(seps, len);
        var i: usize = 0;
        while (i < len) : (i += 1) {
            if ((mask & (@as(u32, 1) << @intCast(i))) != 0) {
                parser.params_sep.set(i);
            }
        }
    } else {
        for (seps[0..len], 0..) |sep, i| {
            if (sep == ':') parser.params_sep.set(i);
        }
    }
}

fn resetIdx(parser: *sgr.Parser) void {
    if (comptime build_options.lib_vt_rust) {
        rust.ghostty_rust_sgr_parser_reset(&parser.idx);
    } else {
        parser.idx = 0;
    }
}

pub fn next(
    parser_: Parser,
    result: *sgr.Attribute.C,
) callconv(lib.calling_conv) bool {
    const wrapper = parser_ orelse return false;
    const parser: *sgr.Parser = &wrapper.parser;
    if (parser.next()) |attr| {
        result.* = attr.cval();
        return true;
    }

    return false;
}

pub fn unknown_full(
    unknown: sgr.Attribute.Unknown.C,
    ptr: ?*[*]const u16,
) callconv(lib.calling_conv) usize {
    if (comptime build_options.lib_vt_rust) {
        return rust.ghostty_rust_sgr_unknown_full(unknown, ptr);
    }

    if (ptr) |p| p.* = unknown.full_ptr;
    return unknown.full_len;
}

pub fn unknown_partial(
    unknown: sgr.Attribute.Unknown.C,
    ptr: ?*[*]const u16,
) callconv(lib.calling_conv) usize {
    if (comptime build_options.lib_vt_rust) {
        return rust.ghostty_rust_sgr_unknown_partial(unknown, ptr);
    }

    if (ptr) |p| p.* = unknown.partial_ptr;
    return unknown.partial_len;
}

pub fn attribute_tag(
    attr: sgr.Attribute.C,
) callconv(lib.calling_conv) sgr.Attribute.Tag {
    if (comptime build_options.lib_vt_rust) {
        return rust.ghostty_rust_sgr_attribute_tag(attr);
    }

    return attr.tag;
}

pub fn attribute_value(
    attr: *sgr.Attribute.C,
) callconv(lib.calling_conv) *sgr.Attribute.CValue {
    if (comptime build_options.lib_vt_rust) {
        return rust.ghostty_rust_sgr_attribute_value(attr);
    }

    return &attr.value;
}

pub fn wasm_alloc_attribute() callconv(lib.calling_conv) *sgr.Attribute.C {
    const alloc = std.heap.wasm_allocator;
    const ptr = alloc.create(sgr.Attribute.C) catch @panic("out of memory");
    return ptr;
}

pub fn wasm_free_attribute(attr: *sgr.Attribute.C) callconv(lib.calling_conv) void {
    const alloc = std.heap.wasm_allocator;
    alloc.destroy(attr);
}

test "alloc" {
    var p: Parser = undefined;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &p,
    ));
    free(p);
}

test "simple params, no seps" {
    var p: Parser = undefined;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &p,
    ));
    defer free(p);

    try testing.expectEqual(Result.success, setParams(
        p,
        &.{1},
        null,
        1,
    ));

    // Set it twice on purpose to make sure we don't leak.
    try testing.expectEqual(Result.success, setParams(
        p,
        &.{1},
        null,
        1,
    ));

    // Verify we get bold
    var attr: sgr.Attribute.C = undefined;
    try testing.expect(next(p, &attr));
    try testing.expectEqual(.bold, attr.tag);

    // Nothing else
    try testing.expect(!next(p, &attr));
}

test "reset replays params" {
    var p: Parser = undefined;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &p,
    ));
    defer free(p);

    try testing.expectEqual(Result.success, setParams(
        p,
        &.{ 1, 3 },
        null,
        2,
    ));

    var attr: sgr.Attribute.C = undefined;
    try testing.expect(next(p, &attr));
    try testing.expectEqual(.bold, attr.tag);

    reset(p);

    try testing.expect(next(p, &attr));
    try testing.expectEqual(.bold, attr.tag);
    try testing.expect(next(p, &attr));
    try testing.expectEqual(.italic, attr.tag);
}

test "colon separators" {
    var p: Parser = undefined;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &p,
    ));
    defer free(p);

    try testing.expectEqual(Result.success, setParams(
        p,
        &.{ 38, 2, 1, 2, 3 },
        "::::",
        5,
    ));

    var attr: sgr.Attribute.C = undefined;
    try testing.expect(next(p, &attr));
    try testing.expectEqual(.direct_color_fg, attr.tag);
}
