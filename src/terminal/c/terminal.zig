const std = @import("std");
const testing = std.testing;
const build_options = @import("terminal_options");
const lib = @import("../lib.zig");
const CAllocator = lib.alloc.Allocator;
pub const ZigTerminal = @import("../Terminal.zig");
const Stream = @import("../stream_terminal.zig").Stream;
const ScreenSet = @import("../ScreenSet.zig");
const PageList = @import("../PageList.zig");
const apc = @import("../apc.zig");
const kitty = @import("../kitty/key.zig");
const kitty_gfx_c = @import("kitty_graphics.zig");
const modes = @import("../modes.zig");
const point = @import("../point.zig");
const size = @import("../size.zig");
const device_attributes = @import("../device_attributes.zig");
const device_status = @import("../device_status.zig");
const size_report = @import("../size_report.zig");
const cell_c = @import("cell.zig");
const row_c = @import("row.zig");
const grid_ref_c = @import("grid_ref.zig");
const grid_ref_tracked_c = @import("grid_ref_tracked.zig");
const selection_c = @import("selection.zig");
const style_c = @import("style.zig");
const color = @import("../color.zig");
const Result = @import("result.zig").Result;

const Handler = @import("../stream_terminal.zig").Handler;

const log = std.log.scoped(.terminal_c);

const rust = if (build_options.lib_vt_rust) struct {
    extern fn ghostty_rust_terminal_new(
        cols: size.CellCountInt,
        rows: size.CellCountInt,
    ) callconv(.c) c_int;

    extern fn ghostty_rust_terminal_reset(has_terminal: bool) callconv(.c) bool;

    extern fn ghostty_rust_terminal_get(
        has_terminal: bool,
        data: c_int,
        has_out: bool,
    ) callconv(.c) c_int;

    extern fn ghostty_rust_terminal_get_scalar(
        data: c_int,
        cols: size.CellCountInt,
        rows: size.CellCountInt,
        cursor_x: size.CellCountInt,
        cursor_y: size.CellCountInt,
        cursor_pending_wrap: bool,
        active_screen: c_int,
        cursor_visible: bool,
        kitty_keyboard_flags: u8,
        mouse_tracking: bool,
        total_rows: usize,
        scrollback_rows: usize,
        width_px: u32,
        height_px: u32,
        out: ?*anyopaque,
    ) callconv(.c) c_int;

    extern fn ghostty_rust_terminal_get_scalar_multi(
        count: usize,
        keys: ?[*]const TerminalData,
        values: ?[*]?*anyopaque,
        out_written: ?*usize,
        cols: size.CellCountInt,
        rows: size.CellCountInt,
        cursor_x: size.CellCountInt,
        cursor_y: size.CellCountInt,
        cursor_pending_wrap: bool,
        active_screen: c_int,
        cursor_visible: bool,
        kitty_keyboard_flags: u8,
        mouse_tracking: bool,
        total_rows: usize,
        scrollback_rows: usize,
        width_px: u32,
        height_px: u32,
    ) callconv(.c) c_int;

    extern fn ghostty_rust_terminal_get_string(
        data: c_int,
        ptr: [*]const u8,
        len: usize,
        out: ?*anyopaque,
    ) callconv(.c) c_int;

    extern fn ghostty_rust_terminal_get_style(
        data: c_int,
        style_: *const style_c.Style,
        out: ?*anyopaque,
    ) callconv(.c) c_int;

    extern fn ghostty_rust_terminal_get_scrollbar(
        data: c_int,
        total: u64,
        offset: u64,
        len: u64,
        out: ?*anyopaque,
    ) callconv(.c) c_int;

    extern fn ghostty_rust_terminal_get_kitty_image(
        data: c_int,
        enabled: bool,
        storage_limit: u64,
        medium_file: bool,
        medium_temp_file: bool,
        medium_shared_mem: bool,
        out: ?*anyopaque,
    ) callconv(.c) c_int;

    extern fn ghostty_rust_terminal_get_color(
        data: c_int,
        has_value: bool,
        r: u8,
        g: u8,
        b: u8,
        out: ?*anyopaque,
    ) callconv(.c) c_int;

    extern fn ghostty_rust_terminal_get_palette(
        data: c_int,
        palette: *const color.PaletteC,
        out: ?*anyopaque,
    ) callconv(.c) c_int;

    extern fn ghostty_rust_terminal_get_pointer(
        data: c_int,
        has_value: bool,
        value: ?*anyopaque,
        out: ?*anyopaque,
    ) callconv(.c) c_int;

    extern fn ghostty_rust_terminal_get_selection(
        data: c_int,
        has_value: bool,
        selection: ?*const selection_c.CSelection,
        out: ?*anyopaque,
    ) callconv(.c) c_int;

    extern fn ghostty_rust_terminal_point_from_grid_ref(
        has_point: bool,
        coord: point.Coordinate,
        out: ?*point.Coordinate,
    ) callconv(.c) c_int;

    extern fn ghostty_rust_terminal_point_from_grid_ref_input(
        has_terminal: bool,
        has_ref: bool,
    ) callconv(.c) c_int;

    extern fn ghostty_rust_terminal_grid_ref(
        has_pin: bool,
        node: ?*anyopaque,
        x: size.CellCountInt,
        y: size.CellCountInt,
        out_ref: ?*grid_ref_c.CGridRef,
    ) callconv(.c) c_int;

    extern fn ghostty_rust_terminal_grid_ref_track_input(
        has_terminal: bool,
        has_out: bool,
    ) callconv(.c) c_int;

    extern fn ghostty_rust_terminal_mode_get(
        has_terminal: bool,
        has_mode: bool,
        value: bool,
        out: *bool,
    ) callconv(.c) c_int;

    extern fn ghostty_rust_terminal_mode_set(
        has_terminal: bool,
        has_mode: bool,
    ) callconv(.c) c_int;

    extern fn ghostty_rust_terminal_resize(
        has_terminal: bool,
        cols: size.CellCountInt,
        rows: size.CellCountInt,
        cell_width_px: u32,
        cell_height_px: u32,
        out_width_px: *u32,
        out_height_px: *u32,
    ) callconv(.c) c_int;

    extern fn ghostty_rust_terminal_set(
        has_terminal: bool,
        option: c_int,
    ) callconv(.c) c_int;

    extern fn ghostty_rust_terminal_set_string(
        value: ?*const lib.String,
        out_ptr: *[*]const u8,
        out_len: *usize,
    ) callconv(.c) c_int;

    extern fn ghostty_rust_terminal_set_rgb(
        value: ?*const color.RGB.C,
        out_has_value: *bool,
        out_rgb: *color.RGB.C,
    ) callconv(.c) c_int;

    extern fn ghostty_rust_terminal_set_palette(
        value: ?*const color.PaletteC,
        out_has_value: *bool,
        out_palette: *[*]const color.RGB.C,
    ) callconv(.c) c_int;

    extern fn ghostty_rust_terminal_set_u64_zero(
        value: ?*const u64,
        out_value: *u64,
    ) callconv(.c) c_int;

    extern fn ghostty_rust_terminal_set_bool_optional(
        value: ?*const bool,
        out_has_value: *bool,
        out_value: *bool,
    ) callconv(.c) c_int;

    extern fn ghostty_rust_terminal_set_usize_optional(
        value: ?*const usize,
        out_has_value: *bool,
        out_value: *usize,
    ) callconv(.c) c_int;
} else struct {};

const rust_owned = if (build_options.terminal_rust_owned) struct {
    extern fn ghostty_rust_terminal_create(
        alloc: ?*const CAllocator,
        cols: size.CellCountInt,
        rows: size.CellCountInt,
        max_scrollback: usize,
    ) callconv(.c) ?*anyopaque;

    extern fn ghostty_rust_terminal_destroy(
        alloc: ?*const CAllocator,
        handle: ?*anyopaque,
    ) callconv(.c) void;

    extern fn ghostty_rust_terminal_write(
        handle: ?*anyopaque,
        ptr: [*]const u8,
        len: usize,
    ) callconv(.c) void;

    extern fn ghostty_rust_terminal_owned_grid_ref(
        handle: ?*anyopaque,
        pt: *const point.Point.C,
        out_ref: ?*grid_ref_c.CGridRef,
    ) callconv(.c) c_int;

    extern fn ghostty_rust_terminal_owned_resize(
        handle: ?*anyopaque,
        alloc: ?*const CAllocator,
        cols: size.CellCountInt,
        rows: size.CellCountInt,
        cell_width_px: u32,
        cell_height_px: u32,
        out_width_px: *u32,
        out_height_px: *u32,
    ) callconv(.c) c_int;

    extern fn ghostty_rust_terminal_owned_reset(handle: ?*anyopaque) callconv(.c) void;

    extern fn ghostty_rust_terminal_owned_get_scalar(
        handle: ?*anyopaque,
        data: c_int,
        out: ?*anyopaque,
    ) callconv(.c) c_int;

    extern fn ghostty_rust_terminal_owned_get_scalar_multi(
        handle: ?*anyopaque,
        count: usize,
        keys: ?[*]const TerminalData,
        values: ?[*]?*anyopaque,
        out_written: ?*usize,
    ) callconv(.c) c_int;

    extern fn ghostty_rust_terminal_owned_point_from_grid_ref(
        handle: ?*anyopaque,
        ref_: ?*const grid_ref_c.CGridRef,
        tag: u8,
        out: ?*point.Coordinate,
    ) callconv(.c) c_int;

    extern fn ghostty_rust_terminal_owned_mode_get(
        handle: ?*anyopaque,
        tag: modes.ModeTag.Backing,
        out: *bool,
    ) callconv(.c) c_int;

    extern fn ghostty_rust_terminal_owned_mode_set(
        handle: ?*anyopaque,
        tag: modes.ModeTag.Backing,
        value: bool,
    ) callconv(.c) c_int;

    extern fn ghostty_rust_terminal_owned_set_string(
        handle: ?*anyopaque,
        alloc: ?*const CAllocator,
        data: c_int,
        value: ?*const lib.String,
    ) callconv(.c) c_int;

    extern fn ghostty_rust_terminal_owned_get_string(
        handle: ?*anyopaque,
        data: c_int,
        out: ?*anyopaque,
    ) callconv(.c) c_int;

    extern fn ghostty_rust_terminal_owned_get_style(
        handle: ?*anyopaque,
        data: c_int,
        out: ?*anyopaque,
    ) callconv(.c) c_int;

    extern fn ghostty_rust_terminal_owned_get_scrollbar(
        handle: ?*anyopaque,
        data: c_int,
        out: ?*anyopaque,
    ) callconv(.c) c_int;

    extern fn ghostty_rust_terminal_owned_get_color(
        handle: ?*anyopaque,
        data: c_int,
        out: ?*anyopaque,
    ) callconv(.c) c_int;

    extern fn ghostty_rust_terminal_owned_set_color(
        handle: ?*anyopaque,
        data: c_int,
        value: ?*const color.RGB.C,
    ) callconv(.c) c_int;

    extern fn ghostty_rust_terminal_owned_set_wrapper(
        handle: ?*anyopaque,
        wrapper: ?*TerminalWrapper,
    ) callconv(.c) void;

    extern fn ghostty_rust_terminal_owned_set_palette(
        handle: ?*anyopaque,
        value: ?*const color.PaletteC,
    ) callconv(.c) c_int;

    extern fn ghostty_rust_terminal_owned_get_palette(
        handle: ?*anyopaque,
        data: c_int,
        out: ?*anyopaque,
    ) callconv(.c) c_int;

    extern fn ghostty_rust_terminal_owned_set_selection(
        handle: ?*anyopaque,
        value: ?*const selection_c.CSelection,
    ) callconv(.c) c_int;

    extern fn ghostty_rust_terminal_owned_get_selection(
        handle: ?*anyopaque,
        out: ?*anyopaque,
    ) callconv(.c) c_int;

    extern fn ghostty_rust_terminal_owned_set_apc_max_bytes(
        handle: ?*anyopaque,
        value: ?*const usize,
    ) callconv(.c) c_int;

    extern fn ghostty_rust_terminal_owned_set_apc_max_bytes_kitty(
        handle: ?*anyopaque,
        value: ?*const usize,
    ) callconv(.c) c_int;

    extern fn ghostty_rust_terminal_owned_set_kitty_image_storage_limit(
        handle: ?*anyopaque,
        value: ?*const u64,
    ) callconv(.c) c_int;

    extern fn ghostty_rust_terminal_owned_set_kitty_image_medium(
        handle: ?*anyopaque,
        option: c_int,
        value: ?*const bool,
    ) callconv(.c) c_int;

    extern fn ghostty_rust_terminal_owned_get_kitty_image(
        handle: ?*anyopaque,
        data: c_int,
        enabled: bool,
        out: ?*anyopaque,
    ) callconv(.c) c_int;

    extern fn ghostty_rust_terminal_owned_set_color_override(
        handle: ?*anyopaque,
        data: c_int,
        value: ?*const color.RGB.C,
    ) callconv(.c) c_int;

    extern fn ghostty_rust_terminal_owned_set_palette_index(
        handle: ?*anyopaque,
        index: u8,
        value: ?*const color.RGB.C,
    ) callconv(.c) c_int;
} else struct {};

fn rustOwnedHandle(wrapper: *TerminalWrapper) ?*anyopaque {
    return switch (wrapper.state) {
        .rust => |r| r.handle,
        .zig => null,
    };
}

fn rustOwnedAlloc(wrapper: *const TerminalWrapper) ?*const CAllocator {
    return switch (wrapper.state) {
        .rust => |r| r.alloc,
        .zig => null,
    };
}

/// Wrapper around terminal state for C API usage. The active backend is either
/// a Zig terminal plus VT stream, or a Rust-owned terminal handle.
const TerminalWrapper = struct {
    state: State,
    effects: Effects = .{},
    tracked_grid_refs: std.AutoArrayHashMapUnmanaged(*grid_ref_tracked_c.TrackedGridRef, void) = .{},

    const State = union(enum) {
        zig: struct {
            terminal: *ZigTerminal,
            stream: Stream,
        },
        rust: struct {
            handle: *anyopaque,
            alloc: ?*const CAllocator,
        },
    };

    fn zigTerminal(self: *const TerminalWrapper) ?*ZigTerminal {
        return switch (self.state) {
            .zig => |s| s.terminal,
            .rust => null,
        };
    }

    fn zigStream(self: *TerminalWrapper) ?*Stream {
        return switch (self.state) {
            .zig => |*s| &s.stream,
            .rust => null,
        };
    }
};

pub fn terminalZig(terminal_: Terminal) ?*ZigTerminal {
    return (terminal_ orelse return null).zigTerminal();
}

pub fn wrapperZig(wrapper: *TerminalWrapper) ?*ZigTerminal {
    return wrapper.zigTerminal();
}

/// C callback state for terminal effects. Trampolines are always
/// installed on the stream handler; they check these fields and
/// no-op when the corresponding callback is null.
const Effects = struct {
    userdata: ?*anyopaque = null,
    write_pty: ?WritePtyFn = null,
    bell: ?BellFn = null,
    color_scheme: ?ColorSchemeFn = null,
    device_attributes_cb: ?DeviceAttributesFn = null,
    enquiry: ?EnquiryFn = null,
    xtversion: ?XtversionFn = null,
    title_changed: ?TitleChangedFn = null,
    size_cb: ?SizeFn = null,

    /// Scratch buffer for DA1 feature codes. The device attributes
    /// trampoline converts C feature codes into this buffer and returns
    /// a slice pointing into it. Storing it here ensures the slice
    /// remains valid after the trampoline returns, since the caller
    /// (`reportDeviceAttributes`) reads it before any re-entrant call.
    da_features_buf: [64]device_attributes.Primary.Feature = undefined,

    /// C function pointer type for the write_pty callback.
    pub const WritePtyFn = *const fn (Terminal, ?*anyopaque, [*]const u8, usize) callconv(lib.calling_conv) void;

    /// C function pointer type for the bell callback.
    pub const BellFn = *const fn (Terminal, ?*anyopaque) callconv(lib.calling_conv) void;

    /// C function pointer type for the color_scheme callback.
    /// Returns true and fills out_scheme if a color scheme is available,
    /// or returns false to silently ignore the query.
    pub const ColorSchemeFn = *const fn (Terminal, ?*anyopaque, *device_status.ColorScheme) callconv(lib.calling_conv) bool;

    /// C function pointer type for the enquiry callback.
    /// Returns the response bytes. The memory must remain valid
    /// until the callback returns.
    pub const EnquiryFn = *const fn (Terminal, ?*anyopaque) callconv(lib.calling_conv) lib.String;

    /// C function pointer type for the xtversion callback.
    /// Returns the version string (e.g. "ghostty 1.2.3"). The memory
    /// must remain valid until the callback returns. An empty string
    /// (len=0) causes the default "libghostty" to be reported.
    pub const XtversionFn = *const fn (Terminal, ?*anyopaque) callconv(lib.calling_conv) lib.String;

    /// C function pointer type for the title_changed callback.
    pub const TitleChangedFn = *const fn (Terminal, ?*anyopaque) callconv(lib.calling_conv) void;

    /// C function pointer type for the size callback.
    /// Returns true and fills out_size if size is available,
    /// or returns false to silently ignore the query.
    pub const SizeFn = *const fn (Terminal, ?*anyopaque, *size_report.Size) callconv(lib.calling_conv) bool;

    /// C function pointer type for the device_attributes callback.
    /// Returns true and fills out_attrs if attributes are available,
    /// or returns false to silently ignore the query.
    pub const DeviceAttributesFn = *const fn (Terminal, ?*anyopaque, *CDeviceAttributes) callconv(lib.calling_conv) bool;

    /// C-compatible device attributes struct.
    /// C: GhosttyDeviceAttributes
    pub const CDeviceAttributes = extern struct {
        primary: Primary,
        secondary: Secondary,
        tertiary: Tertiary,

        pub const Primary = extern struct {
            conformance_level: u16,
            features: [64]u16,
            num_features: usize,
        };

        pub const Secondary = extern struct {
            device_type: u16,
            firmware_version: u16,
            rom_cartridge: u16,
        };

        pub const Tertiary = extern struct {
            unit_id: u32,
        };
    };

    fn wrapperFromHandler(handler: *Handler) *TerminalWrapper {
        return @ptrCast(@alignCast(handler.c_wrapper.?));
    }

    fn writePtyTrampoline(handler: *Handler, data: [:0]const u8) void {
        const wrapper = wrapperFromHandler(handler);
        const func = wrapper.effects.write_pty orelse return;
        func(@ptrCast(wrapper), wrapper.effects.userdata, data.ptr, data.len);
    }

    fn bellTrampoline(handler: *Handler) void {
        const wrapper = wrapperFromHandler(handler);
        const func = wrapper.effects.bell orelse return;
        func(@ptrCast(wrapper), wrapper.effects.userdata);
    }

    fn colorSchemeTrampoline(handler: *Handler) ?device_status.ColorScheme {
        const wrapper = wrapperFromHandler(handler);
        const func = wrapper.effects.color_scheme orelse return null;
        var scheme: device_status.ColorScheme = undefined;
        if (func(@ptrCast(wrapper), wrapper.effects.userdata, &scheme)) return scheme;
        return null;
    }

    fn deviceAttributesTrampoline(handler: *Handler) device_attributes.Attributes {
        const wrapper = wrapperFromHandler(handler);
        const func = wrapper.effects.device_attributes_cb orelse return .{};

        // Get our attributes from the callback.
        var c_attrs: CDeviceAttributes = undefined;
        if (!func(@ptrCast(wrapper), wrapper.effects.userdata, &c_attrs)) return .{};

        // Note below we use a lot of enumFromInt but its always safe
        // because all our types are non-exhaustive enums.

        const n: usize = @min(c_attrs.primary.num_features, 64);
        for (0..n) |i| wrapper.effects.da_features_buf[i] = @enumFromInt(c_attrs.primary.features[i]);

        return .{
            .primary = .{
                .conformance_level = @enumFromInt(c_attrs.primary.conformance_level),
                .features = wrapper.effects.da_features_buf[0..n],
            },
            .secondary = .{
                .device_type = @enumFromInt(c_attrs.secondary.device_type),
                .firmware_version = c_attrs.secondary.firmware_version,
                .rom_cartridge = c_attrs.secondary.rom_cartridge,
            },
            .tertiary = .{
                .unit_id = c_attrs.tertiary.unit_id,
            },
        };
    }

    fn enquiryTrampoline(handler: *Handler) []const u8 {
        const wrapper = wrapperFromHandler(handler);
        const func = wrapper.effects.enquiry orelse return "";
        const result = func(@ptrCast(wrapper), wrapper.effects.userdata);
        if (result.len == 0) return "";
        return result.ptr[0..result.len];
    }

    fn xtversionTrampoline(handler: *Handler) []const u8 {
        const wrapper = wrapperFromHandler(handler);
        const func = wrapper.effects.xtversion orelse return "";
        const result = func(@ptrCast(wrapper), wrapper.effects.userdata);
        if (result.len == 0) return "";
        return result.ptr[0..result.len];
    }

    fn titleChangedTrampoline(handler: *Handler) void {
        const wrapper = wrapperFromHandler(handler);
        const func = wrapper.effects.title_changed orelse return;
        func(@ptrCast(wrapper), wrapper.effects.userdata);
    }

    fn sizeTrampoline(handler: *Handler) ?size_report.Size {
        const wrapper = wrapperFromHandler(handler);
        const func = wrapper.effects.size_cb orelse return null;
        var s: size_report.Size = undefined;
        if (func(@ptrCast(wrapper), wrapper.effects.userdata, &s)) return s;
        return null;
    }
};

fn wrapperWritePty(wrapper: *TerminalWrapper, data: []const u8) void {
    const func = wrapper.effects.write_pty orelse return;
    func(@ptrCast(wrapper), wrapper.effects.userdata, data.ptr, data.len);
}

fn deviceAttributesFromWrapper(wrapper: *TerminalWrapper) device_attributes.Attributes {
    const func = wrapper.effects.device_attributes_cb orelse return .{};
    var c_attrs: Effects.CDeviceAttributes = undefined;
    if (!func(@ptrCast(wrapper), wrapper.effects.userdata, &c_attrs)) return .{};

    const n: usize = @min(c_attrs.primary.num_features, 64);
    for (0..n) |i| wrapper.effects.da_features_buf[i] = @enumFromInt(c_attrs.primary.features[i]);

    return .{
        .primary = .{
            .conformance_level = @enumFromInt(c_attrs.primary.conformance_level),
            .features = wrapper.effects.da_features_buf[0..n],
        },
        .secondary = .{
            .device_type = @enumFromInt(c_attrs.secondary.device_type),
            .firmware_version = c_attrs.secondary.firmware_version,
            .rom_cartridge = c_attrs.secondary.rom_cartridge,
        },
        .tertiary = .{
            .unit_id = c_attrs.tertiary.unit_id,
        },
    };
}

/// Write PTY output for a rust-owned terminal via wrapper effects.
pub export fn ghostty_terminal_wrapper_write_pty(
    wrapper: ?*TerminalWrapper,
    ptr: [*]const u8,
    len: usize,
) callconv(.c) void {
    const w = wrapper orelse return;
    if (len == 0) return;
    wrapperWritePty(w, ptr[0..len]);
}

/// Ring the terminal bell via wrapper effects.
pub export fn ghostty_terminal_wrapper_bell(wrapper: ?*TerminalWrapper) callconv(.c) void {
    const w = wrapper orelse return;
    const func = w.effects.bell orelse return;
    func(@ptrCast(w), w.effects.userdata);
}

/// Notify that the window title changed via wrapper effects.
pub export fn ghostty_terminal_wrapper_title_changed(wrapper: ?*TerminalWrapper) callconv(.c) void {
    const w = wrapper orelse return;
    const func = w.effects.title_changed orelse return;
    func(@ptrCast(w), w.effects.userdata);
}

/// Respond to ENQ using wrapper effects.
pub export fn ghostty_terminal_wrapper_report_enquiry(wrapper: ?*TerminalWrapper) callconv(.c) void {
    const w = wrapper orelse return;
    const func = w.effects.enquiry orelse return;
    if (w.effects.write_pty == null) return;
    const result = func(@ptrCast(w), w.effects.userdata);
    if (result.len == 0) return;
    wrapperWritePty(w, result.ptr[0..result.len]);
}

/// Respond to XTVERSION using wrapper effects.
pub export fn ghostty_terminal_wrapper_report_xtversion(wrapper: ?*TerminalWrapper) callconv(.c) void {
    const w = wrapper orelse return;
    if (w.effects.write_pty == null) return;
    const version = if (w.effects.xtversion) |func| func(@ptrCast(w), w.effects.userdata) else lib.String{ .ptr = "", .len = 0 };
    var buf: [288]u8 = undefined;
    const resp = std.fmt.bufPrint(
        &buf,
        "\x1BP>|{s}\x1B\\",
        .{if (version.len > 0) version.ptr[0..version.len] else "libghostty"},
    ) catch return;
    wrapperWritePty(w, resp);
}

/// Respond to device attribute queries using wrapper effects.
pub export fn ghostty_terminal_wrapper_report_device_attributes(
    wrapper: ?*TerminalWrapper,
    req: u8,
) callconv(.c) void {
    const w = wrapper orelse return;
    if (w.effects.write_pty == null) return;
    const req_enum: device_attributes.Req = @enumFromInt(req);
    const attrs = deviceAttributesFromWrapper(w);
    var buf: [256]u8 = undefined;
    var writer: std.Io.Writer = .fixed(&buf);
    attrs.encode(req_enum, &writer) catch return;
    const len = writer.buffered().len;
    wrapperWritePty(w, buf[0..len]);
}

/// Fill size report values from the size callback.
pub export fn ghostty_terminal_wrapper_query_size(
    wrapper: ?*TerminalWrapper,
    out: *size_report.Size,
) callconv(.c) bool {
    const w = wrapper orelse return false;
    const func = w.effects.size_cb orelse return false;
    return func(@ptrCast(w), w.effects.userdata, out);
}

/// Respond to color scheme DSR using wrapper effects.
pub export fn ghostty_terminal_wrapper_report_color_scheme(wrapper: ?*TerminalWrapper) callconv(.c) void {
    const w = wrapper orelse return;
    const func = w.effects.color_scheme orelse return;
    if (w.effects.write_pty == null) return;
    var scheme: device_status.ColorScheme = undefined;
    if (!func(@ptrCast(w), w.effects.userdata, &scheme)) return;
    const resp: []const u8 = switch (scheme) {
        .dark => "\x1B[?997;1n",
        .light => "\x1B[?997;2n",
    };
    wrapperWritePty(w, resp);
}

/// C: GhosttyTerminal
pub const Terminal = ?*TerminalWrapper;

pub fn zigTerminal(terminal_: Terminal) ?*ZigTerminal {
    return (terminal_ orelse return null).zigTerminal();
}

/// C: GhosttyTerminalOptions
pub const Options = extern struct {
    cols: size.CellCountInt,
    rows: size.CellCountInt,
    max_scrollback: usize,
};

const NewError = error{
    InvalidValue,
    OutOfMemory,
};

pub fn new(
    alloc_: ?*const CAllocator,
    result: *Terminal,
    opts: Options,
) callconv(lib.calling_conv) Result {
    if (comptime build_options.terminal_rust_owned) {
        result.* = newOwned(alloc_, opts) catch |err| {
            result.* = null;
            return switch (err) {
                error.InvalidValue => .invalid_value,
                error.OutOfMemory => .out_of_memory,
            };
        };
        return .success;
    }

    if (comptime build_options.lib_vt_rust) {
        const validation: Result = @enumFromInt(rust.ghostty_rust_terminal_new(
            opts.cols,
            opts.rows,
        ));
        if (validation != .success) {
            result.* = null;
            return validation;
        }
    }

    result.* = new_(alloc_, opts) catch |err| {
        result.* = null;
        return switch (err) {
            error.InvalidValue => .invalid_value,
            error.OutOfMemory => .out_of_memory,
        };
    };

    return .success;
}

fn newOwned(
    alloc_: ?*const CAllocator,
    opts: Options,
) NewError!*TerminalWrapper {
    if (opts.cols == 0 or opts.rows == 0) return error.InvalidValue;

    const alloc = lib.alloc.default(alloc_);
    const handle = rust_owned.ghostty_rust_terminal_create(
        alloc_,
        opts.cols,
        opts.rows,
        opts.max_scrollback,
    ) orelse return error.OutOfMemory;

    const wrapper = alloc.create(TerminalWrapper) catch {
        rust_owned.ghostty_rust_terminal_destroy(alloc_, handle);
        return error.OutOfMemory;
    };

    wrapper.* = .{
        .state = .{ .rust = .{
            .handle = handle,
            .alloc = alloc_,
        } },
        .effects = .{},
        .tracked_grid_refs = .{},
    };

    rust_owned.ghostty_rust_terminal_owned_set_wrapper(handle, wrapper);

    return wrapper;
}

fn new_(
    alloc_: ?*const CAllocator,
    opts: Options,
) NewError!*TerminalWrapper {
    if (opts.cols == 0 or opts.rows == 0) return error.InvalidValue;

    const alloc = lib.alloc.default(alloc_);
    const t = alloc.create(ZigTerminal) catch
        return error.OutOfMemory;
    errdefer alloc.destroy(t);

    const wrapper = alloc.create(TerminalWrapper) catch
        return error.OutOfMemory;
    errdefer alloc.destroy(wrapper);

    // Setup our terminal
    t.* = try .init(alloc, .{
        .cols = opts.cols,
        .rows = opts.rows,
        .max_scrollback = opts.max_scrollback,
    });
    errdefer t.deinit(alloc);

    // Setup our stream with trampolines always installed so that
    // setting C callbacks at any time takes effect immediately.
    var handler: Stream.Handler = t.vtHandler();
    handler.c_wrapper = wrapper;
    handler.effects = .{
        .write_pty = &Effects.writePtyTrampoline,
        .bell = &Effects.bellTrampoline,
        .color_scheme = &Effects.colorSchemeTrampoline,
        .device_attributes = &Effects.deviceAttributesTrampoline,
        .enquiry = &Effects.enquiryTrampoline,
        .xtversion = &Effects.xtversionTrampoline,
        .title_changed = &Effects.titleChangedTrampoline,
        .size = &Effects.sizeTrampoline,
    };

    wrapper.* = .{
        .state = .{ .zig = .{
            .terminal = t,
            .stream = .initAlloc(alloc, handler),
        } },
        .effects = .{},
        .tracked_grid_refs = .{},
    };

    return wrapper;
}

pub fn vt_write(
    terminal_: Terminal,
    ptr: [*]const u8,
    len: usize,
) callconv(lib.calling_conv) void {
    const wrapper = terminal_ orelse return;
    switch (wrapper.state) {
        .zig => |*s| s.stream.nextSlice(ptr[0..len]),
        .rust => |r| {
            if (comptime build_options.terminal_rust_owned) {
                rust_owned.ghostty_rust_terminal_write(r.handle, ptr, len);
            }
        },
    }
}

/// C: GhosttyTerminalOption
pub const Option = enum(c_int) {
    userdata = 0,
    write_pty = 1,
    bell = 2,
    enquiry = 3,
    xtversion = 4,
    title_changed = 5,
    size_cb = 6,
    color_scheme = 7,
    device_attributes = 8,
    title = 9,
    pwd = 10,
    color_foreground = 11,
    color_background = 12,
    color_cursor = 13,
    color_palette = 14,
    kitty_image_storage_limit = 15,
    kitty_image_medium_file = 16,
    kitty_image_medium_temp_file = 17,
    kitty_image_medium_shared_mem = 18,
    apc_max_bytes = 19,
    apc_max_bytes_kitty = 20,
    selection = 21,

    /// Input type expected for setting the option.
    pub fn InType(comptime self: Option) type {
        return switch (self) {
            .userdata => ?*const anyopaque,
            .write_pty => ?Effects.WritePtyFn,
            .bell => ?Effects.BellFn,
            .color_scheme => ?Effects.ColorSchemeFn,
            .device_attributes => ?Effects.DeviceAttributesFn,
            .enquiry => ?Effects.EnquiryFn,
            .xtversion => ?Effects.XtversionFn,
            .title_changed => ?Effects.TitleChangedFn,
            .size_cb => ?Effects.SizeFn,
            .title, .pwd => ?*const lib.String,
            .color_foreground, .color_background, .color_cursor => ?*const color.RGB.C,
            .color_palette => ?*const color.PaletteC,
            .kitty_image_storage_limit => ?*const u64,
            .kitty_image_medium_file,
            .kitty_image_medium_temp_file,
            .kitty_image_medium_shared_mem,
            => ?*const bool,
            .apc_max_bytes, .apc_max_bytes_kitty => ?*const usize,
            .selection => ?*const selection_c.CSelection,
        };
    }
};

pub fn set(
    terminal_: Terminal,
    option: Option,
    value: ?*const anyopaque,
) callconv(lib.calling_conv) Result {
    if (comptime build_options.lib_vt_rust) {
        const result: Result = @enumFromInt(rust.ghostty_rust_terminal_set(
            terminal_ != null,
            @intFromEnum(option),
        ));
        if (result != .success) return result;
    } else if (comptime std.debug.runtime_safety) {
        _ = std.meta.intToEnum(Option, @intFromEnum(option)) catch {
            log.warn("terminal_set invalid option value={d}", .{@intFromEnum(option)});
            return .invalid_value;
        };
    }

    const wrapper = terminal_ orelse return .invalid_value;

    return switch (option) {
        inline else => |comptime_option| setTyped(
            wrapper,
            comptime_option,
            @ptrCast(@alignCast(value)),
        ),
    };
}

fn setTyped(
    wrapper: *TerminalWrapper,
    comptime option: Option,
    value: option.InType(),
) Result {
    switch (option) {
        .userdata => wrapper.effects.userdata = @constCast(value),
        .write_pty => wrapper.effects.write_pty = value,
        .bell => wrapper.effects.bell = value,
        .color_scheme => wrapper.effects.color_scheme = value,
        .device_attributes => wrapper.effects.device_attributes_cb = value,
        .enquiry => wrapper.effects.enquiry = value,
        .xtversion => wrapper.effects.xtversion = value,
        .title_changed => wrapper.effects.title_changed = value,
        .size_cb => wrapper.effects.size_cb = value,
        .title => {
            const str = if (comptime build_options.lib_vt_rust) str: {
                var ptr: [*]const u8 = undefined;
                var len: usize = undefined;
                const result: Result = @enumFromInt(rust.ghostty_rust_terminal_set_string(
                    value,
                    &ptr,
                    &len,
                ));
                if (result != .success) return result;
                break :str ptr[0..len];
            } else if (value) |v| v.ptr[0..v.len] else "";
            if (comptime build_options.terminal_rust_owned) {
                if (rustOwnedHandle(wrapper)) |handle| {
                    return @enumFromInt(rust_owned.ghostty_rust_terminal_owned_set_string(
                        handle,
                        rustOwnedAlloc(wrapper),
                        @intFromEnum(TerminalData.title),
                        value,
                    ));
                }
            }
            const t = wrapper.zigTerminal() orelse return .invalid_value;
            t.setTitle(str) catch return .out_of_memory;
        },
        .pwd => {
            const str = if (comptime build_options.lib_vt_rust) str: {
                var ptr: [*]const u8 = undefined;
                var len: usize = undefined;
                const result: Result = @enumFromInt(rust.ghostty_rust_terminal_set_string(
                    value,
                    &ptr,
                    &len,
                ));
                if (result != .success) return result;
                break :str ptr[0..len];
            } else if (value) |v| v.ptr[0..v.len] else "";
            if (comptime build_options.terminal_rust_owned) {
                if (rustOwnedHandle(wrapper)) |handle| {
                    return @enumFromInt(rust_owned.ghostty_rust_terminal_owned_set_string(
                        handle,
                        rustOwnedAlloc(wrapper),
                        @intFromEnum(TerminalData.pwd),
                        value,
                    ));
                }
            }
            const t = wrapper.zigTerminal() orelse return .invalid_value;
            t.setPwd(str) catch return .out_of_memory;
        },
        .color_foreground => {
            if (comptime build_options.terminal_rust_owned) {
                if (rustOwnedHandle(wrapper)) |handle| {
                    return @enumFromInt(rust_owned.ghostty_rust_terminal_owned_set_color(
                        handle,
                        @intFromEnum(TerminalData.color_foreground),
                        value,
                    ));
                }
            }
            var rgb: ?color.RGB = null;
            const result = decodeSetRgb(value, &rgb);
            if (result != .success) return result;
            const t = wrapper.zigTerminal() orelse return .invalid_value;
            t.colors.foreground.default = rgb;
            t.flags.dirty.palette = true;
        },
        .color_background => {
            if (comptime build_options.terminal_rust_owned) {
                if (rustOwnedHandle(wrapper)) |handle| {
                    return @enumFromInt(rust_owned.ghostty_rust_terminal_owned_set_color(
                        handle,
                        @intFromEnum(TerminalData.color_background),
                        value,
                    ));
                }
            }
            var rgb: ?color.RGB = null;
            const result = decodeSetRgb(value, &rgb);
            if (result != .success) return result;
            const t = wrapper.zigTerminal() orelse return .invalid_value;
            t.colors.background.default = rgb;
            t.flags.dirty.palette = true;
        },
        .color_cursor => {
            if (comptime build_options.terminal_rust_owned) {
                if (rustOwnedHandle(wrapper)) |handle| {
                    return @enumFromInt(rust_owned.ghostty_rust_terminal_owned_set_color(
                        handle,
                        @intFromEnum(TerminalData.color_cursor),
                        value,
                    ));
                }
            }
            var rgb: ?color.RGB = null;
            const result = decodeSetRgb(value, &rgb);
            if (result != .success) return result;
            const t = wrapper.zigTerminal() orelse return .invalid_value;
            t.colors.cursor.default = rgb;
            t.flags.dirty.palette = true;
        },
        .color_palette => {
            if (comptime build_options.terminal_rust_owned) {
                if (rustOwnedHandle(wrapper)) |handle| {
                    return @enumFromInt(rust_owned.ghostty_rust_terminal_owned_set_palette(
                        handle,
                        value,
                    ));
                }
            }
            var palette: color.Palette = undefined;
            const result = decodeSetPalette(value, &palette);
            if (result != .success) return result;
            const t = wrapper.zigTerminal() orelse return .invalid_value;
            t.colors.palette.changeDefault(palette);
            t.flags.dirty.palette = true;
        },
        .kitty_image_storage_limit => {
            if (comptime !build_options.kitty_graphics) return .success;
            if (comptime build_options.terminal_rust_owned) {
                if (rustOwnedHandle(wrapper)) |handle| {
                    return @enumFromInt(rust_owned.ghostty_rust_terminal_owned_set_kitty_image_storage_limit(
                        handle,
                        value,
                    ));
                }
            }
            var limit_u64: u64 = undefined;
            const result = decodeSetU64Zero(value, &limit_u64);
            if (result != .success) return result;
            const limit: usize = @intCast(limit_u64);
            const t = wrapper.zigTerminal() orelse return .invalid_value;
            var it = t.screens.all.iterator();
            while (it.next()) |entry| {
                const screen = entry.value.*;
                screen.kitty_images.setLimit(screen.alloc, screen, limit) catch return .out_of_memory;
            }
        },
        .kitty_image_medium_file,
        .kitty_image_medium_temp_file,
        .kitty_image_medium_shared_mem,
        => {
            if (comptime !build_options.kitty_graphics) return .success;
            if (comptime build_options.terminal_rust_owned) {
                if (rustOwnedHandle(wrapper)) |handle| {
                    return @enumFromInt(rust_owned.ghostty_rust_terminal_owned_set_kitty_image_medium(
                        handle,
                        @intFromEnum(option),
                        value,
                    ));
                }
            }
            var val: bool = undefined;
            var has_value: bool = undefined;
            const result = decodeSetBoolOptional(value, &has_value, &val);
            if (result != .success) return result;
            if (!has_value) return .success;
            const t = wrapper.zigTerminal() orelse return .invalid_value;
            var it = t.screens.all.iterator();
            while (it.next()) |entry| {
                const screen = entry.value.*;
                switch (option) {
                    .kitty_image_medium_file => screen.kitty_images.image_limits.file = val,
                    .kitty_image_medium_temp_file => screen.kitty_images.image_limits.temporary_file = val,
                    .kitty_image_medium_shared_mem => screen.kitty_images.image_limits.shared_memory = val,
                    else => unreachable,
                }
            }
        },
        .apc_max_bytes => {
            var has_value: bool = undefined;
            var max_bytes: usize = undefined;
            const result = decodeSetUsizeOptional(value, &has_value, &max_bytes);
            if (result != .success) return result;
            if (comptime build_options.terminal_rust_owned) {
                if (rustOwnedHandle(wrapper)) |handle| {
                    return @enumFromInt(rust_owned.ghostty_rust_terminal_owned_set_apc_max_bytes(
                        handle,
                        value,
                    ));
                }
            }
            const stream = wrapper.zigStream() orelse return .invalid_value;
            stream.handler.apc_handler.max_bytes = if (has_value)
                .initFull(max_bytes)
            else
                .{};
        },
        .apc_max_bytes_kitty => {
            var has_value: bool = undefined;
            var max_bytes: usize = undefined;
            const result = decodeSetUsizeOptional(value, &has_value, &max_bytes);
            if (result != .success) return result;
            if (comptime build_options.terminal_rust_owned) {
                if (rustOwnedHandle(wrapper)) |handle| {
                    return @enumFromInt(rust_owned.ghostty_rust_terminal_owned_set_apc_max_bytes_kitty(
                        handle,
                        value,
                    ));
                }
            }
            const stream = wrapper.zigStream() orelse return .invalid_value;
            if (has_value) {
                stream.handler.apc_handler.max_bytes.put(.kitty, max_bytes);
            } else {
                stream.handler.apc_handler.max_bytes.remove(.kitty);
            }
        },
        .selection => {
            if (comptime build_options.terminal_rust_owned) {
                if (rustOwnedHandle(wrapper)) |handle| {
                    return @enumFromInt(rust_owned.ghostty_rust_terminal_owned_set_selection(
                        handle,
                        value,
                    ));
                }
            }
            if (value) |ptr| {
                const sel = ptr.toZig() orelse return .invalid_value;
                const t = wrapper.zigTerminal() orelse return .invalid_value;
                t.screens.active.select(sel) catch return .out_of_memory;
            } else {
                const t = wrapper.zigTerminal() orelse return .invalid_value;
                t.screens.active.clearSelection();
            }
        },
    }
    return .success;
}

fn decodeSetPalette(value: ?*const color.PaletteC, out: *color.Palette) Result {
    if (comptime build_options.lib_vt_rust) {
        var has_value: bool = undefined;
        var palette_ptr: [*]const color.RGB.C = undefined;
        const result: Result = @enumFromInt(rust.ghostty_rust_terminal_set_palette(
            value,
            &has_value,
            &palette_ptr,
        ));
        if (result != .success) return result;

        out.* = if (has_value) palette: {
            const palette_c: *const color.PaletteC = @ptrCast(palette_ptr);
            break :palette color.paletteZval(palette_c);
        } else color.default;
    } else {
        out.* = if (value) |v| color.paletteZval(v) else color.default;
    }

    return .success;
}

fn decodeSetUsizeOptional(value: ?*const usize, out_has_value: *bool, out_value: *usize) Result {
    if (comptime build_options.lib_vt_rust) {
        return @enumFromInt(rust.ghostty_rust_terminal_set_usize_optional(
            value,
            out_has_value,
            out_value,
        ));
    }

    if (value) |v| {
        out_has_value.* = true;
        out_value.* = v.*;
    } else {
        out_has_value.* = false;
    }
    return .success;
}

fn decodeSetU64Zero(value: ?*const u64, out: *u64) Result {
    if (comptime build_options.lib_vt_rust) {
        return @enumFromInt(rust.ghostty_rust_terminal_set_u64_zero(value, out));
    }

    out.* = if (value) |v| v.* else 0;
    return .success;
}

fn decodeSetBoolOptional(value: ?*const bool, out_has_value: *bool, out_value: *bool) Result {
    if (comptime build_options.lib_vt_rust) {
        return @enumFromInt(rust.ghostty_rust_terminal_set_bool_optional(
            value,
            out_has_value,
            out_value,
        ));
    }

    if (value) |v| {
        out_has_value.* = true;
        out_value.* = v.*;
    } else {
        out_has_value.* = false;
    }
    return .success;
}

fn decodeSetRgb(value: ?*const color.RGB.C, out: *?color.RGB) Result {
    if (comptime build_options.lib_vt_rust) {
        var has_value: bool = undefined;
        var rgb: color.RGB.C = undefined;
        const result: Result = @enumFromInt(rust.ghostty_rust_terminal_set_rgb(
            value,
            &has_value,
            &rgb,
        ));
        if (result != .success) return result;

        out.* = if (has_value) .fromC(rgb) else null;
    } else {
        out.* = if (value) |v| .fromC(v.*) else null;
    }

    return .success;
}

/// C: GhosttyDeviceAttributes
pub const DeviceAttributes = Effects.CDeviceAttributes;

/// C: GhosttyTerminalScrollViewport
pub const ScrollViewport = ZigTerminal.ScrollViewport.C;

pub fn scroll_viewport(
    terminal_: Terminal,
    behavior: ScrollViewport,
) callconv(lib.calling_conv) void {
    const t: *ZigTerminal = terminalZig(terminal_) orelse return;
    t.scrollViewport(switch (behavior.tag) {
        .top => .top,
        .bottom => .bottom,
        .delta => .{ .delta = behavior.value.delta },
    });
}

pub fn resize(
    terminal_: Terminal,
    cols: size.CellCountInt,
    rows: size.CellCountInt,
    cell_width_px: u32,
    cell_height_px: u32,
) callconv(lib.calling_conv) Result {
    var width_px: u32 = undefined;
    var height_px: u32 = undefined;

    const wrapper = terminal_ orelse {
        if (comptime build_options.lib_vt_rust) {
            return @enumFromInt(rust.ghostty_rust_terminal_resize(
                false,
                cols,
                rows,
                cell_width_px,
                cell_height_px,
                &width_px,
                &height_px,
            ));
        }
        return .invalid_value;
    };

    if (comptime build_options.terminal_rust_owned) {
        if (rustOwnedHandle(wrapper)) |handle| {
            const result: Result = @enumFromInt(rust_owned.ghostty_rust_terminal_owned_resize(
                handle,
                rustOwnedAlloc(wrapper),
                cols,
                rows,
                cell_width_px,
                cell_height_px,
                &width_px,
                &height_px,
            ));
            if (result != .success) return result;

            const synchronized_output: modes.ModeTag.Backing = @bitCast(modes.ModeTag{
                .value = 2026,
                .ansi = false,
            });
            _ = rust_owned.ghostty_rust_terminal_owned_mode_set(handle, synchronized_output, false);

            const in_band_size_reports: modes.ModeTag.Backing = @bitCast(modes.ModeTag{
                .value = 2048,
                .ansi = false,
            });
            var in_band: bool = undefined;
            if (rust_owned.ghostty_rust_terminal_owned_mode_get(handle, in_band_size_reports, &in_band) == @intFromEnum(Result.success) and
                in_band)
                in_band:
            {
                const func = wrapper.effects.write_pty orelse break :in_band;

                var buf: [1024]u8 = undefined;
                var writer: std.Io.Writer = .fixed(&buf);
                size_report.encode(&writer, .mode_2048, .{
                    .rows = rows,
                    .columns = cols,
                    .cell_width = cell_width_px,
                    .cell_height = cell_height_px,
                }) catch break :in_band;

                const data = writer.buffered();
                func(@ptrCast(wrapper), wrapper.effects.userdata, data.ptr, data.len);
            }

            return .success;
        }
    }

    if (comptime build_options.lib_vt_rust) {
        const result: Result = @enumFromInt(rust.ghostty_rust_terminal_resize(
            true,
            cols,
            rows,
            cell_width_px,
            cell_height_px,
            &width_px,
            &height_px,
        ));
        if (result != .success) return result;
    } else {
        if (cols == 0 or rows == 0) return .invalid_value;
        width_px = std.math.mul(u32, cols, cell_width_px) catch std.math.maxInt(u32);
        height_px = std.math.mul(u32, rows, cell_height_px) catch std.math.maxInt(u32);
    }

    const t = wrapper.zigTerminal() orelse return .invalid_value;
    t.resize(t.gpa(), cols, rows) catch return .out_of_memory;

    // Update pixel sizes
    t.width_px = width_px;
    t.height_px = height_px;

    // Disable synchronized output mode so that we show changes
    // immediately for a resize. This is allowed by the spec.
    t.modes.set(.synchronized_output, false);

    // If we have in-band size reporting enabled, send a report.
    if (t.modes.get(.in_band_size_reports)) in_band: {
        const func = wrapper.effects.write_pty orelse break :in_band;

        var buf: [1024]u8 = undefined;
        var writer: std.Io.Writer = .fixed(&buf);
        size_report.encode(&writer, .mode_2048, .{
            .rows = rows,
            .columns = cols,
            .cell_width = cell_width_px,
            .cell_height = cell_height_px,
        }) catch break :in_band;

        const data = writer.buffered();
        func(@ptrCast(wrapper), wrapper.effects.userdata, data.ptr, data.len);
    }

    return .success;
}

pub fn reset(terminal_: Terminal) callconv(lib.calling_conv) void {
    if (comptime build_options.lib_vt_rust) {
        if (!rust.ghostty_rust_terminal_reset(terminal_ != null)) return;
    }

    const wrapper = terminal_ orelse return;
    if (comptime build_options.terminal_rust_owned) {
        if (rustOwnedHandle(wrapper)) |handle| {
            rust_owned.ghostty_rust_terminal_owned_reset(handle);
            return;
        }
    }

    const t: *ZigTerminal = wrapper.zigTerminal() orelse return;
    t.fullReset();
}

pub fn mode_get(
    terminal_: Terminal,
    tag: modes.ModeTag.Backing,
    out_value: *bool,
) callconv(lib.calling_conv) Result {
    const wrapper = terminal_ orelse {
        if (comptime build_options.lib_vt_rust) {
            return @enumFromInt(rust.ghostty_rust_terminal_mode_get(
                false,
                false,
                false,
                out_value,
            ));
        }
        return .invalid_value;
    };
    const t: *ZigTerminal = wrapper.zigTerminal() orelse {
        if (comptime build_options.terminal_rust_owned) {
            if (rustOwnedHandle(wrapper)) |handle| {
                return @enumFromInt(rust_owned.ghostty_rust_terminal_owned_mode_get(
                    handle,
                    tag,
                    out_value,
                ));
            }
        }
        if (comptime build_options.lib_vt_rust) {
            return @enumFromInt(rust.ghostty_rust_terminal_mode_get(
                true,
                false,
                false,
                out_value,
            ));
        }
        return .invalid_value;
    };
    const mode_tag: modes.ModeTag = @bitCast(tag);
    const mode = modes.modeFromInt(mode_tag.value, mode_tag.ansi) orelse {
        if (comptime build_options.lib_vt_rust) {
            return @enumFromInt(rust.ghostty_rust_terminal_mode_get(
                true,
                false,
                false,
                out_value,
            ));
        }
        return .invalid_value;
    };
    if (comptime build_options.lib_vt_rust) {
        return @enumFromInt(rust.ghostty_rust_terminal_mode_get(
            true,
            true,
            t.modes.get(mode),
            out_value,
        ));
    }
    out_value.* = t.modes.get(mode);
    return .success;
}

pub fn mode_set(
    terminal_: Terminal,
    tag: modes.ModeTag.Backing,
    value: bool,
) callconv(lib.calling_conv) Result {
    const wrapper = terminal_ orelse {
        if (comptime build_options.lib_vt_rust) {
            return @enumFromInt(rust.ghostty_rust_terminal_mode_set(false, false));
        }
        return .invalid_value;
    };
    const t: *ZigTerminal = wrapper.zigTerminal() orelse {
        if (comptime build_options.terminal_rust_owned) {
            if (rustOwnedHandle(wrapper)) |handle| {
                return @enumFromInt(rust_owned.ghostty_rust_terminal_owned_mode_set(
                    handle,
                    tag,
                    value,
                ));
            }
        }
        if (comptime build_options.lib_vt_rust) {
            return @enumFromInt(rust.ghostty_rust_terminal_mode_set(true, false));
        }
        return .invalid_value;
    };
    const mode_tag: modes.ModeTag = @bitCast(tag);
    const mode = modes.modeFromInt(mode_tag.value, mode_tag.ansi) orelse {
        if (comptime build_options.lib_vt_rust) {
            return @enumFromInt(rust.ghostty_rust_terminal_mode_set(true, false));
        }
        return .invalid_value;
    };
    if (comptime build_options.lib_vt_rust) {
        const result: Result = @enumFromInt(rust.ghostty_rust_terminal_mode_set(true, true));
        if (result != .success) return result;
    }
    t.modes.set(mode, value);
    return .success;
}

/// C: GhosttyKittyGraphics
pub const KittyGraphics = kitty_gfx_c.KittyGraphics;

/// C: GhosttyTerminalScreen
pub const TerminalScreen = ScreenSet.Key;

/// C: GhosttyTerminalScrollbar
pub const TerminalScrollbar = PageList.Scrollbar.C;

/// C: GhosttyTerminalData
pub const TerminalData = enum(c_int) {
    invalid = 0,
    cols = 1,
    rows = 2,
    cursor_x = 3,
    cursor_y = 4,
    cursor_pending_wrap = 5,
    active_screen = 6,
    cursor_visible = 7,
    kitty_keyboard_flags = 8,
    scrollbar = 9,
    cursor_style = 10,
    mouse_tracking = 11,
    title = 12,
    pwd = 13,
    total_rows = 14,
    scrollback_rows = 15,
    width_px = 16,
    height_px = 17,
    color_foreground = 18,
    color_background = 19,
    color_cursor = 20,
    color_palette = 21,
    color_foreground_default = 22,
    color_background_default = 23,
    color_cursor_default = 24,
    color_palette_default = 25,
    kitty_image_storage_limit = 26,
    kitty_image_medium_file = 27,
    kitty_image_medium_temp_file = 28,
    kitty_image_medium_shared_mem = 29,
    kitty_graphics = 30,
    selection = 31,

    /// Output type expected for querying the data of the given kind.
    pub fn OutType(comptime self: TerminalData) type {
        return switch (self) {
            .invalid => void,
            .cols, .rows, .cursor_x, .cursor_y => size.CellCountInt,
            .cursor_pending_wrap, .cursor_visible, .mouse_tracking => bool,
            .active_screen => TerminalScreen,
            .kitty_keyboard_flags => u8,
            .scrollbar => TerminalScrollbar,
            .cursor_style => style_c.Style,
            .title, .pwd => lib.String,
            .total_rows, .scrollback_rows => usize,
            .width_px, .height_px => u32,
            .color_foreground,
            .color_background,
            .color_cursor,
            .color_foreground_default,
            .color_background_default,
            .color_cursor_default,
            => color.RGB.C,
            .color_palette, .color_palette_default => color.PaletteC,
            .kitty_image_storage_limit => u64,
            .kitty_image_medium_file,
            .kitty_image_medium_temp_file,
            .kitty_image_medium_shared_mem,
            => bool,
            .kitty_graphics => KittyGraphics,
            .selection => selection_c.CSelection,
        };
    }
};

pub fn get(
    terminal_: Terminal,
    data: TerminalData,
    out: ?*anyopaque,
) callconv(lib.calling_conv) Result {
    if (comptime std.debug.runtime_safety) {
        _ = std.meta.intToEnum(TerminalData, @intFromEnum(data)) catch {
            log.warn("terminal_get invalid data value={d}", .{@intFromEnum(data)});
            return .invalid_value;
        };
    }

    if (comptime build_options.lib_vt_rust) {
        const result: Result = @enumFromInt(rust.ghostty_rust_terminal_get(
            terminal_ != null,
            @intFromEnum(data),
            out != null,
        ));
        if (result != .success) return result;
    }

    return switch (data) {
        .invalid => .invalid_value,
        inline else => |comptime_data| getTyped(
            terminal_,
            comptime_data,
            @ptrCast(@alignCast(out)),
        ),
    };
}

pub fn get_multi(
    terminal_: Terminal,
    count: usize,
    keys: ?[*]const TerminalData,
    values: ?[*]?*anyopaque,
    out_written: ?*usize,
) callconv(lib.calling_conv) Result {
    const k = keys orelse return .invalid_value;
    const v = values orelse return .invalid_value;
    if (count == 0) {
        if (out_written) |w| w.* = 0;
        return .success;
    }

    if (comptime build_options.lib_vt_rust) {
        var scalar_only = true;
        for (0..count) |i| {
            switch (k[i]) {
                .invalid,
                .cols,
                .rows,
                .cursor_x,
                .cursor_y,
                .cursor_pending_wrap,
                .active_screen,
                .cursor_visible,
                .kitty_keyboard_flags,
                .mouse_tracking,
                .total_rows,
                .scrollback_rows,
                .width_px,
                .height_px,
                => {},
                else => {
                    scalar_only = false;
                    break;
                },
            }
        }

        if (scalar_only) {
            const wrapper = terminal_ orelse return .invalid_value;
            if (comptime build_options.terminal_rust_owned) {
                if (rustOwnedHandle(wrapper)) |handle| {
                    return @enumFromInt(rust_owned.ghostty_rust_terminal_owned_get_scalar_multi(
                        handle,
                        count,
                        k,
                        v,
                        out_written,
                    ));
                }
            }
            const t: *ZigTerminal = wrapper.zigTerminal() orelse return .invalid_value;
            return @enumFromInt(rust.ghostty_rust_terminal_get_scalar_multi(
                count,
                k,
                v,
                out_written,
                t.cols,
                t.rows,
                t.screens.active.cursor.x,
                t.screens.active.cursor.y,
                t.screens.active.cursor.pending_wrap,
                @intFromEnum(t.screens.active_key),
                t.modes.get(.cursor_visible),
                @as(u8, t.screens.active.kitty_keyboard.current().int()),
                t.modes.get(.mouse_event_x10) or
                    t.modes.get(.mouse_event_normal) or
                    t.modes.get(.mouse_event_button) or
                    t.modes.get(.mouse_event_any),
                t.screens.active.pages.total_rows,
                t.screens.active.pages.total_rows - t.rows,
                t.width_px,
                t.height_px,
            ));
        }
    }

    for (0..count) |i| {
        const result = get(terminal_, k[i], v[i]);
        if (result != .success) {
            if (out_written) |w| w.* = i;
            return result;
        }
    }
    if (out_written) |w| w.* = count;
    return .success;
}

fn getTyped(
    terminal_: Terminal,
    comptime data: TerminalData,
    out: *data.OutType(),
) Result {
    const wrapper = terminal_ orelse return .invalid_value;
    if (comptime build_options.terminal_rust_owned) {
        if (rustOwnedHandle(wrapper)) |handle| {
            switch (data) {
                .cols,
                .rows,
                .cursor_x,
                .cursor_y,
                .cursor_pending_wrap,
                .active_screen,
                .cursor_visible,
                .kitty_keyboard_flags,
                .mouse_tracking,
                .total_rows,
                .scrollback_rows,
                .width_px,
                .height_px,
                => return @enumFromInt(rust_owned.ghostty_rust_terminal_owned_get_scalar(
                    handle,
                    @intFromEnum(data),
                    @ptrCast(out),
                )),
                .title,
                .pwd,
                => return @enumFromInt(rust_owned.ghostty_rust_terminal_owned_get_string(
                    handle,
                    @intFromEnum(data),
                    @ptrCast(out),
                )),
                .cursor_style => return @enumFromInt(rust_owned.ghostty_rust_terminal_owned_get_style(
                    handle,
                    @intFromEnum(data),
                    @ptrCast(out),
                )),
                .scrollbar => return @enumFromInt(rust_owned.ghostty_rust_terminal_owned_get_scrollbar(
                    handle,
                    @intFromEnum(data),
                    @ptrCast(out),
                )),
                .color_foreground,
                .color_background,
                .color_cursor,
                .color_foreground_default,
                .color_background_default,
                .color_cursor_default,
                => return @enumFromInt(rust_owned.ghostty_rust_terminal_owned_get_color(
                    handle,
                    @intFromEnum(data),
                    @ptrCast(out),
                )),
                .color_palette,
                .color_palette_default,
                => return @enumFromInt(rust_owned.ghostty_rust_terminal_owned_get_palette(
                    handle,
                    @intFromEnum(data),
                    @ptrCast(out),
                )),
                .kitty_image_storage_limit,
                .kitty_image_medium_file,
                .kitty_image_medium_temp_file,
                .kitty_image_medium_shared_mem,
                => return @enumFromInt(rust_owned.ghostty_rust_terminal_owned_get_kitty_image(
                    handle,
                    @intFromEnum(data),
                    comptime build_options.kitty_graphics,
                    @ptrCast(out),
                )),
                .selection => return @enumFromInt(rust_owned.ghostty_rust_terminal_owned_get_selection(
                    handle,
                    @ptrCast(out),
                )),
                else => return .invalid_value,
            }
        }
    }

    const t: *ZigTerminal = wrapper.zigTerminal() orelse return .invalid_value;
    if (comptime build_options.lib_vt_rust) {
        switch (data) {
            .cols,
            .rows,
            .cursor_x,
            .cursor_y,
            .cursor_pending_wrap,
            .active_screen,
            .cursor_visible,
            .kitty_keyboard_flags,
            .mouse_tracking,
            .total_rows,
            .scrollback_rows,
            .width_px,
            .height_px,
            => return @enumFromInt(rust.ghostty_rust_terminal_get_scalar(
                @intFromEnum(data),
                t.cols,
                t.rows,
                t.screens.active.cursor.x,
                t.screens.active.cursor.y,
                t.screens.active.cursor.pending_wrap,
                @intFromEnum(t.screens.active_key),
                t.modes.get(.cursor_visible),
                @as(u8, t.screens.active.kitty_keyboard.current().int()),
                t.modes.get(.mouse_event_x10) or
                    t.modes.get(.mouse_event_normal) or
                    t.modes.get(.mouse_event_button) or
                    t.modes.get(.mouse_event_any),
                t.screens.active.pages.total_rows,
                t.screens.active.pages.total_rows - t.rows,
                t.width_px,
                t.height_px,
                @ptrCast(out),
            )),
            else => {},
        }

        switch (data) {
            .title,
            .pwd,
            => {
                const value = switch (data) {
                    .title => t.getTitle() orelse "",
                    .pwd => t.getPwd() orelse "",
                    else => unreachable,
                };
                return @enumFromInt(rust.ghostty_rust_terminal_get_string(
                    @intFromEnum(data),
                    value.ptr,
                    value.len,
                    @ptrCast(out),
                ));
            },
            else => {},
        }

        switch (data) {
            .cursor_style => {
                const value: style_c.Style = .fromStyle(t.screens.active.cursor.style);
                return @enumFromInt(rust.ghostty_rust_terminal_get_style(
                    @intFromEnum(data),
                    &value,
                    @ptrCast(out),
                ));
            },
            else => {},
        }

        switch (data) {
            .scrollbar => {
                const value = t.screens.active.pages.scrollbar().cval();
                return @enumFromInt(rust.ghostty_rust_terminal_get_scrollbar(
                    @intFromEnum(data),
                    value.total,
                    value.offset,
                    value.len,
                    @ptrCast(out),
                ));
            },
            else => {},
        }

        switch (data) {
            .kitty_image_storage_limit,
            .kitty_image_medium_file,
            .kitty_image_medium_temp_file,
            .kitty_image_medium_shared_mem,
            => {
                if (comptime build_options.kitty_graphics) {
                    return @enumFromInt(rust.ghostty_rust_terminal_get_kitty_image(
                        @intFromEnum(data),
                        true,
                        @intCast(t.screens.active.kitty_images.total_limit),
                        t.screens.active.kitty_images.image_limits.file,
                        t.screens.active.kitty_images.image_limits.temporary_file,
                        t.screens.active.kitty_images.image_limits.shared_memory,
                        @ptrCast(out),
                    ));
                }

                return @enumFromInt(rust.ghostty_rust_terminal_get_kitty_image(
                    @intFromEnum(data),
                    false,
                    0,
                    false,
                    false,
                    false,
                    @ptrCast(out),
                ));
            },
            else => {},
        }

        switch (data) {
            .color_foreground,
            .color_background,
            .color_cursor,
            .color_foreground_default,
            .color_background_default,
            .color_cursor_default,
            => {
                const value: ?color.RGB.C = switch (data) {
                    .color_foreground => if (t.colors.foreground.get()) |v| v.cval() else null,
                    .color_background => if (t.colors.background.get()) |v| v.cval() else null,
                    .color_cursor => if (t.colors.cursor.get()) |v| v.cval() else null,
                    .color_foreground_default => if (t.colors.foreground.default) |v| v.cval() else null,
                    .color_background_default => if (t.colors.background.default) |v| v.cval() else null,
                    .color_cursor_default => if (t.colors.cursor.default) |v| v.cval() else null,
                    else => unreachable,
                };
                return @enumFromInt(rust.ghostty_rust_terminal_get_color(
                    @intFromEnum(data),
                    value != null,
                    if (value) |v| v.r else 0,
                    if (value) |v| v.g else 0,
                    if (value) |v| v.b else 0,
                    @ptrCast(out),
                ));
            },
            else => {},
        }

        switch (data) {
            .color_palette,
            .color_palette_default,
            => {
                const value: color.PaletteC = switch (data) {
                    .color_palette => color.paletteCval(&t.colors.palette.current),
                    .color_palette_default => color.paletteCval(&t.colors.palette.original),
                    else => unreachable,
                };
                return @enumFromInt(rust.ghostty_rust_terminal_get_palette(
                    @intFromEnum(data),
                    &value,
                    @ptrCast(out),
                ));
            },
            else => {},
        }

        switch (data) {
            .kitty_graphics => {
                return @enumFromInt(rust.ghostty_rust_terminal_get_pointer(
                    @intFromEnum(data),
                    comptime build_options.kitty_graphics,
                    if (comptime build_options.kitty_graphics)
                        @ptrCast(&t.screens.active.kitty_images)
                    else
                        null,
                    @ptrCast(out),
                ));
            },
            else => {},
        }

        switch (data) {
            .selection => {
                const maybe_value: ?selection_c.CSelection = if (t.screens.active.selection) |sel|
                    selection_c.CSelection.fromZig(sel)
                else
                    null;
                return @enumFromInt(rust.ghostty_rust_terminal_get_selection(
                    @intFromEnum(data),
                    maybe_value != null,
                    if (maybe_value) |*value| value else null,
                    @ptrCast(out),
                ));
            },
            else => {},
        }
    }

    switch (data) {
        .invalid => return .invalid_value,
        .cols => out.* = t.cols,
        .rows => out.* = t.rows,
        .cursor_x => out.* = t.screens.active.cursor.x,
        .cursor_y => out.* = t.screens.active.cursor.y,
        .cursor_pending_wrap => out.* = t.screens.active.cursor.pending_wrap,
        .active_screen => out.* = t.screens.active_key,
        .cursor_visible => out.* = t.modes.get(.cursor_visible),
        .kitty_keyboard_flags => out.* = @as(u8, t.screens.active.kitty_keyboard.current().int()),
        .scrollbar => out.* = t.screens.active.pages.scrollbar().cval(),
        .cursor_style => out.* = .fromStyle(t.screens.active.cursor.style),
        .mouse_tracking => out.* = t.modes.get(.mouse_event_x10) or
            t.modes.get(.mouse_event_normal) or
            t.modes.get(.mouse_event_button) or
            t.modes.get(.mouse_event_any),
        .title => {
            const title = t.getTitle() orelse "";
            out.* = .{ .ptr = title.ptr, .len = title.len };
        },
        .pwd => {
            const pwd = t.getPwd() orelse "";
            out.* = .{ .ptr = pwd.ptr, .len = pwd.len };
        },
        .total_rows => out.* = t.screens.active.pages.total_rows,
        .scrollback_rows => out.* = t.screens.active.pages.total_rows - t.rows,
        .width_px => out.* = t.width_px,
        .height_px => out.* = t.height_px,
        .color_foreground => out.* = (t.colors.foreground.get() orelse return .no_value).cval(),
        .color_background => out.* = (t.colors.background.get() orelse return .no_value).cval(),
        .color_cursor => out.* = (t.colors.cursor.get() orelse return .no_value).cval(),
        .color_foreground_default => out.* = (t.colors.foreground.default orelse return .no_value).cval(),
        .color_background_default => out.* = (t.colors.background.default orelse return .no_value).cval(),
        .color_cursor_default => out.* = (t.colors.cursor.default orelse return .no_value).cval(),
        .color_palette => out.* = color.paletteCval(&t.colors.palette.current),
        .color_palette_default => out.* = color.paletteCval(&t.colors.palette.original),
        .kitty_image_storage_limit => {
            if (comptime !build_options.kitty_graphics) return .no_value;
            out.* = @intCast(t.screens.active.kitty_images.total_limit);
        },
        .kitty_image_medium_file => {
            if (comptime !build_options.kitty_graphics) return .no_value;
            out.* = t.screens.active.kitty_images.image_limits.file;
        },
        .kitty_image_medium_temp_file => {
            if (comptime !build_options.kitty_graphics) return .no_value;
            out.* = t.screens.active.kitty_images.image_limits.temporary_file;
        },
        .kitty_image_medium_shared_mem => {
            if (comptime !build_options.kitty_graphics) return .no_value;
            out.* = t.screens.active.kitty_images.image_limits.shared_memory;
        },
        .kitty_graphics => {
            if (comptime !build_options.kitty_graphics) return .no_value;
            out.* = &t.screens.active.kitty_images;
        },
        .selection => out.* = selection_c.CSelection.fromZig(
            t.screens.active.selection orelse return .no_value,
        ),
    }

    return .success;
}

pub fn grid_ref(
    terminal_: Terminal,
    pt: point.Point.C,
    out_ref: ?*grid_ref_c.CGridRef,
) callconv(lib.calling_conv) Result {
    const wrapper = terminal_ orelse return .invalid_value;
    if (comptime build_options.terminal_rust_owned) {
        if (rustOwnedHandle(wrapper)) |handle| {
            return @enumFromInt(rust_owned.ghostty_rust_terminal_owned_grid_ref(
                handle,
                &pt,
                out_ref,
            ));
        }
    }

    const t: *ZigTerminal = wrapper.zigTerminal() orelse return .invalid_value;
    const zig_pt: point.Point = .fromC(pt);
    const p = t.screens.active.pages.pin(zig_pt);
    if (comptime build_options.lib_vt_rust) {
        return @enumFromInt(rust.ghostty_rust_terminal_grid_ref(
            p != null,
            if (p) |pin| @ptrCast(pin.node) else null,
            if (p) |pin| pin.x else 0,
            if (p) |pin| pin.y else 0,
            out_ref,
        ));
    }

    const pin = p orelse return .invalid_value;
    if (out_ref) |out| out.* = grid_ref_c.CGridRef.fromPin(pin);
    return .success;
}

pub fn grid_ref_track(
    terminal_: Terminal,
    pt: point.Point.C,
    out_ref: ?*grid_ref_tracked_c.CTrackedGridRef,
) callconv(lib.calling_conv) Result {
    if (comptime build_options.lib_vt_rust) {
        const result: Result = @enumFromInt(rust.ghostty_rust_terminal_grid_ref_track_input(
            terminal_ != null,
            out_ref != null,
        ));
        if (result != .success) return result;
    }

    const wrapper = terminal_ orelse return .invalid_value;
    const out = out_ref orelse return .invalid_value;
    out.* = null;

    const t: *ZigTerminal = wrapper.zigTerminal() orelse return .invalid_value;
    const list = &t.screens.active.pages;
    const p = list.pin(.fromC(pt)) orelse return .invalid_value;
    const tracked_pin = list.trackPin(p) catch return .out_of_memory;

    const alloc = t.gpa();
    const ref = alloc.create(grid_ref_tracked_c.TrackedGridRef) catch {
        list.untrackPin(tracked_pin);
        return .out_of_memory;
    };
    ref.* = .{
        .alloc = alloc,
        .terminal = wrapper,
        .screen_key = t.screens.active_key,
        .screen_generation = t.screens.generation(t.screens.active_key),
        .pin = tracked_pin,
    };

    // Store the tracked ref in the terminal so that when we free
    // the terminal the tracked ref can be detached safely.
    wrapper.tracked_grid_refs.putNoClobber(
        alloc,
        ref,
        {},
    ) catch {
        list.untrackPin(tracked_pin);
        alloc.destroy(ref);
        return .out_of_memory;
    };

    out.* = ref;
    return .success;
}

pub fn point_from_grid_ref(
    terminal_: Terminal,
    ref_: ?*const grid_ref_c.CGridRef,
    tag: point.Tag,
    out: ?*point.Coordinate,
) callconv(lib.calling_conv) Result {
    if (comptime build_options.lib_vt_rust) {
        const result: Result = @enumFromInt(rust.ghostty_rust_terminal_point_from_grid_ref_input(
            terminal_ != null,
            ref_ != null,
        ));
        if (result != .success) return result;
    }

    const wrapper = terminal_ orelse return .invalid_value;
    const ref = ref_ orelse return .invalid_value;
    if (comptime build_options.terminal_rust_owned) {
        if (rustOwnedHandle(wrapper)) |handle| {
            return @enumFromInt(rust_owned.ghostty_rust_terminal_owned_point_from_grid_ref(
                handle,
                ref,
                @intCast(@intFromEnum(tag)),
                out,
            ));
        }
    }

    const t: *ZigTerminal = wrapper.zigTerminal() orelse return .invalid_value;
    const p = ref.toPin() orelse return .invalid_value;
    const pt = t.screens.active.pages.pointFromPin(tag, p);
    if (comptime build_options.lib_vt_rust) {
        return @enumFromInt(rust.ghostty_rust_terminal_point_from_grid_ref(
            pt != null,
            if (pt) |v| v.coord() else .{},
            out,
        ));
    }

    const value = pt orelse return .no_value;
    if (out) |o| o.* = value.coord();
    return .success;
}

/// Clear pwd and title buffers (called from the Rust port's terminal.reset).
pub fn clear_pwd_and_title(terminal_: Terminal) callconv(lib.calling_conv) void {
    const t = terminalZig(terminal_) orelse return;
    t.pwd.clearRetainingCapacity();
    t.title.clearRetainingCapacity();
}

/// Return pwd items pointer and length (called from the Rust port's formatter).
pub fn pwd_items(terminal_: Terminal, out_ptr: *[*]const u8, out_len: *usize) callconv(lib.calling_conv) void {
    const pwd = if (terminalZig(terminal_)) |zt| (zt.getPwd() orelse "") else "";
    out_ptr.* = pwd.ptr;
    out_len.* = pwd.len;
}

pub fn free(terminal_: Terminal) callconv(lib.calling_conv) void {
    const wrapper = terminal_ orelse return;
    const alloc = switch (wrapper.state) {
        .zig => |*s| blk: {
            const gpa = s.terminal.gpa();
            for (wrapper.tracked_grid_refs.keys()) |ref| ref.terminal = null;
            wrapper.tracked_grid_refs.deinit(gpa);
            s.stream.deinit();
            s.terminal.deinit(gpa);
            gpa.destroy(s.terminal);
            break :blk gpa;
        },
        .rust => |r| blk: {
            if (comptime build_options.terminal_rust_owned) {
                rust_owned.ghostty_rust_terminal_destroy(r.alloc, r.handle);
            }
            break :blk lib.alloc.default(r.alloc);
        },
    };
    alloc.destroy(wrapper);
}

/// Returns the Zig terminal for tests that need in-memory inspection. When
/// `-Dterminal-rust-owned=true`, returns null so callers can `orelse return`.
inline fn zigTerminalForTest(t: Terminal) ?*ZigTerminal {
    if (comptime build_options.terminal_rust_owned) return null;
    return t.?.zigTerminal();
}

test "new/free" {
    var t: Terminal = null;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &t,
        .{
            .cols = 80,
            .rows = 24,
            .max_scrollback = 10_000,
        },
    ));

    try testing.expect(t != null);
    free(t);
}

test "new invalid value" {
    var t: Terminal = null;

    try testing.expectEqual(Result.invalid_value, new(
        &lib.alloc.test_allocator,
        &t,
        .{
            .cols = 0,
            .rows = 24,
            .max_scrollback = 10_000,
        },
    ));
    try testing.expect(t == null);

    try testing.expectEqual(Result.invalid_value, new(
        &lib.alloc.test_allocator,
        &t,
        .{
            .cols = 80,
            .rows = 0,
            .max_scrollback = 10_000,
        },
    ));
    try testing.expect(t == null);
}

test "free null" {
    free(null);
}

test "scroll_viewport" {
    var t: Terminal = null;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &t,
        .{
            .cols = 5,
            .rows = 2,
            .max_scrollback = 10_000,
        },
    ));
    defer free(t);

    const zt = zigTerminalForTest(t) orelse return;

    // Write "hello" on the first line
    vt_write(t, "hello", 5);

    // Push "hello" into scrollback with 3 newlines (index = ESC D)
    vt_write(t, "\x1bD\x1bD\x1bD", 6);
    {
        // Viewport should be empty now since hello scrolled off
        const str = try zt.plainString(testing.allocator);
        defer testing.allocator.free(str);
        try testing.expectEqualStrings("", str);
    }

    // Scroll to top: "hello" should be visible again
    scroll_viewport(t, .{ .tag = .top, .value = undefined });
    {
        const str = try zt.plainString(testing.allocator);
        defer testing.allocator.free(str);
        try testing.expectEqualStrings("hello", str);
    }

    // Scroll to bottom: viewport should be empty again
    scroll_viewport(t, .{ .tag = .bottom, .value = undefined });
    {
        const str = try zt.plainString(testing.allocator);
        defer testing.allocator.free(str);
        try testing.expectEqualStrings("", str);
    }

    // Scroll up by delta to bring "hello" back into view
    scroll_viewport(t, .{ .tag = .delta, .value = .{ .delta = -3 } });
    {
        const str = try zt.plainString(testing.allocator);
        defer testing.allocator.free(str);
        try testing.expectEqualStrings("hello", str);
    }
}

test "scroll_viewport null" {
    scroll_viewport(null, .{ .tag = .top, .value = undefined });
}

test "reset" {
    var t: Terminal = null;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &t,
        .{
            .cols = 80,
            .rows = 24,
            .max_scrollback = 10_000,
        },
    ));
    defer free(t);

    vt_write(t, "Hello", 5);
    reset(t);

    const str = try (zigTerminalForTest(t) orelse return).plainString(testing.allocator);
    defer testing.allocator.free(str);
    try testing.expectEqualStrings("", str);
}

test "reset null" {
    reset(null);
}

test "resize" {
    var t: Terminal = null;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &t,
        .{
            .cols = 80,
            .rows = 24,
            .max_scrollback = 10_000,
        },
    ));
    defer free(t);

    try testing.expectEqual(Result.success, resize(t, 40, 12, 9, 18));
    const zt = zigTerminalForTest(t) orelse return;
    try testing.expectEqual(40, zt.cols);
    try testing.expectEqual(12, zt.rows);
}

test "resize null" {
    try testing.expectEqual(Result.invalid_value, resize(null, 80, 24, 9, 18));
}

test "resize invalid value" {
    var t: Terminal = null;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &t,
        .{
            .cols = 80,
            .rows = 24,
            .max_scrollback = 10_000,
        },
    ));
    defer free(t);

    try testing.expectEqual(Result.invalid_value, resize(t, 0, 24, 9, 18));
    try testing.expectEqual(Result.invalid_value, resize(t, 80, 0, 9, 18));
}

test "resize saturates pixel dimensions" {
    var t: Terminal = null;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &t,
        .{
            .cols = 80,
            .rows = 24,
            .max_scrollback = 10_000,
        },
    ));
    defer free(t);

    try testing.expectEqual(Result.success, resize(
        t,
        2,
        2,
        std.math.maxInt(u32),
        std.math.maxInt(u32),
    ));
    const zt = zigTerminalForTest(t) orelse return;
    try testing.expectEqual(std.math.maxInt(u32), zt.width_px);
    try testing.expectEqual(std.math.maxInt(u32), zt.height_px);
}

test "mode_get and mode_set" {
    var t: Terminal = null;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &t,
        .{
            .cols = 80,
            .rows = 24,
            .max_scrollback = 0,
        },
    ));
    defer free(t);

    var value: bool = undefined;

    // DEC mode 25 (cursor_visible) defaults to true
    const cursor_visible: modes.ModeTag.Backing = @bitCast(modes.ModeTag{ .value = 25, .ansi = false });
    try testing.expectEqual(Result.success, mode_get(t, cursor_visible, &value));
    try testing.expect(value);

    // Set it to false
    try testing.expectEqual(Result.success, mode_set(t, cursor_visible, false));
    try testing.expectEqual(Result.success, mode_get(t, cursor_visible, &value));
    try testing.expect(!value);

    // ANSI mode 4 (insert) defaults to false
    const insert: modes.ModeTag.Backing = @bitCast(modes.ModeTag{ .value = 4, .ansi = true });
    try testing.expectEqual(Result.success, mode_get(t, insert, &value));
    try testing.expect(!value);

    try testing.expectEqual(Result.success, mode_set(t, insert, true));
    try testing.expectEqual(Result.success, mode_get(t, insert, &value));
    try testing.expect(value);
}

test "mode_get null" {
    var value: bool = undefined;
    const tag: modes.ModeTag.Backing = @bitCast(modes.ModeTag{ .value = 25, .ansi = false });
    try testing.expectEqual(Result.invalid_value, mode_get(null, tag, &value));
}

test "mode_set null" {
    const tag: modes.ModeTag.Backing = @bitCast(modes.ModeTag{ .value = 25, .ansi = false });
    try testing.expectEqual(Result.invalid_value, mode_set(null, tag, true));
}

test "mode_get unknown mode" {
    var t: Terminal = null;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &t,
        .{
            .cols = 80,
            .rows = 24,
            .max_scrollback = 0,
        },
    ));
    defer free(t);

    var value: bool = undefined;
    const unknown: modes.ModeTag.Backing = @bitCast(modes.ModeTag{ .value = 9999, .ansi = false });
    try testing.expectEqual(Result.invalid_value, mode_get(t, unknown, &value));
}

test "mode_set unknown mode" {
    var t: Terminal = null;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &t,
        .{
            .cols = 80,
            .rows = 24,
            .max_scrollback = 0,
        },
    ));
    defer free(t);

    const unknown: modes.ModeTag.Backing = @bitCast(modes.ModeTag{ .value = 9999, .ansi = false });
    try testing.expectEqual(Result.invalid_value, mode_set(t, unknown, true));
}

test "vt_write" {
    var t: Terminal = null;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &t,
        .{
            .cols = 80,
            .rows = 24,
            .max_scrollback = 10_000,
        },
    ));
    defer free(t);

    vt_write(t, "Hello", 5);

    const str = try (zigTerminalForTest(t) orelse return).plainString(testing.allocator);
    defer testing.allocator.free(str);
    try testing.expectEqualStrings("Hello", str);
}

test "vt_write split escape sequence" {
    var t: Terminal = null;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &t,
        .{
            .cols = 80,
            .rows = 24,
            .max_scrollback = 10_000,
        },
    ));
    defer free(t);

    // Write "Hello" in bold by splitting the CSI bold sequence across two writes.
    // ESC [ 1 m  = bold on, ESC [ 0 m = reset
    // Split ESC from the rest of the CSI sequence.
    vt_write(t, "Hello \x1b", 7);
    vt_write(t, "[1mBold\x1b[0m", 10);

    const str = try (zigTerminalForTest(t) orelse return).plainString(testing.allocator);
    defer testing.allocator.free(str);
    // If the escape sequence leaked, we'd see "[1mBold" as literal text.
    try testing.expectEqualStrings("Hello Bold", str);
}

test "vt_write split combining mark after base at right edge" {
    var t: Terminal = null;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &t,
        .{
            .cols = 2,
            .rows = 2,
            .max_scrollback = 0,
        },
    ));
    defer free(t);

    // Put "å" in the final column, then send its combining low line in a
    // separate write so the mark arrives while the cursor has a pending wrap.
    vt_write(t, "xå", 3);
    vt_write(t, "\xcc\xb2", 2);

    const str = try (zigTerminalForTest(t) orelse return).plainString(testing.allocator);
    defer testing.allocator.free(str);
    try testing.expectEqualStrings("xå̲", str);
}

test "get cols and rows" {
    var t: Terminal = null;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &t,
        .{
            .cols = 80,
            .rows = 24,
            .max_scrollback = 0,
        },
    ));
    defer free(t);

    var cols: size.CellCountInt = undefined;
    var rows: size.CellCountInt = undefined;
    try testing.expectEqual(Result.success, get(t, .cols, @ptrCast(&cols)));
    try testing.expectEqual(Result.success, get(t, .rows, @ptrCast(&rows)));
    try testing.expectEqual(80, cols);
    try testing.expectEqual(24, rows);
}

test "get cursor position" {
    var t: Terminal = null;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &t,
        .{
            .cols = 80,
            .rows = 24,
            .max_scrollback = 0,
        },
    ));
    defer free(t);

    vt_write(t, "Hello", 5);

    var x: size.CellCountInt = undefined;
    var y: size.CellCountInt = undefined;
    try testing.expectEqual(Result.success, get(t, .cursor_x, @ptrCast(&x)));
    try testing.expectEqual(Result.success, get(t, .cursor_y, @ptrCast(&y)));
    try testing.expectEqual(5, x);
    try testing.expectEqual(0, y);
}

test "get cursor style" {
    var t: Terminal = null;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &t,
        .{
            .cols = 80,
            .rows = 24,
            .max_scrollback = 0,
        },
    ));
    defer free(t);

    var cursor_style: style_c.Style = undefined;
    try testing.expectEqual(Result.success, get(t, .cursor_style, @ptrCast(&cursor_style)));
    try testing.expect(style_c.style_is_default(&cursor_style));

    vt_write(t, "\x1b[1;3;4m", 8);
    try testing.expectEqual(Result.success, get(t, .cursor_style, @ptrCast(&cursor_style)));
    try testing.expectEqual(@sizeOf(style_c.Style), cursor_style.size);
    try testing.expectEqual(style_c.ColorTag.none, cursor_style.fg_color.tag);
    try testing.expectEqual(style_c.ColorTag.none, cursor_style.bg_color.tag);
    try testing.expect(cursor_style.bold);
    try testing.expect(cursor_style.italic);
    try testing.expectEqual(@as(c_int, 1), cursor_style.underline);
}

test "get cursor pending wrap" {
    var t: Terminal = null;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &t,
        .{
            .cols = 2,
            .rows = 2,
            .max_scrollback = 0,
        },
    ));
    defer free(t);

    var pending: bool = undefined;
    try testing.expectEqual(Result.success, get(t, .cursor_pending_wrap, @ptrCast(&pending)));
    try testing.expect(!pending);

    vt_write(t, "ab", 2);
    try testing.expectEqual(Result.success, get(t, .cursor_pending_wrap, @ptrCast(&pending)));
    try testing.expect(pending);
}

test "get scalar dimensions" {
    var t: Terminal = null;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &t,
        .{
            .cols = 10,
            .rows = 4,
            .max_scrollback = 10_000,
        },
    ));
    defer free(t);

    try testing.expectEqual(Result.success, resize(t, 12, 5, 7, 9));

    var total_rows: usize = undefined;
    var scrollback_rows: usize = undefined;
    var width_px: u32 = undefined;
    var height_px: u32 = undefined;
    try testing.expectEqual(Result.success, get(t, .total_rows, @ptrCast(&total_rows)));
    try testing.expectEqual(Result.success, get(t, .scrollback_rows, @ptrCast(&scrollback_rows)));
    try testing.expectEqual(Result.success, get(t, .width_px, @ptrCast(&width_px)));
    try testing.expectEqual(Result.success, get(t, .height_px, @ptrCast(&height_px)));
    try testing.expect(total_rows >= 5);
    try testing.expectEqual(total_rows - 5, scrollback_rows);
    try testing.expectEqual(@as(u32, 84), width_px);
    try testing.expectEqual(@as(u32, 45), height_px);
}

test "get null" {
    var cols: size.CellCountInt = undefined;
    try testing.expectEqual(Result.invalid_value, get(null, .cols, @ptrCast(&cols)));
}

test "get cursor_visible" {
    var t: Terminal = null;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &t,
        .{
            .cols = 80,
            .rows = 24,
            .max_scrollback = 0,
        },
    ));
    defer free(t);

    var visible: bool = undefined;
    try testing.expectEqual(Result.success, get(t, .cursor_visible, @ptrCast(&visible)));
    try testing.expect(visible);

    // DEC mode 25 controls cursor visibility
    const cursor_visible_mode: modes.ModeTag.Backing = @bitCast(modes.ModeTag{ .value = 25, .ansi = false });
    try testing.expectEqual(Result.success, mode_set(t, cursor_visible_mode, false));
    try testing.expectEqual(Result.success, get(t, .cursor_visible, @ptrCast(&visible)));
    try testing.expect(!visible);
}

test "get active_screen" {
    var t: Terminal = null;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &t,
        .{
            .cols = 80,
            .rows = 24,
            .max_scrollback = 0,
        },
    ));
    defer free(t);

    var screen: TerminalScreen = undefined;
    try testing.expectEqual(Result.success, get(t, .active_screen, @ptrCast(&screen)));
    try testing.expectEqual(.primary, screen);
}

test "get kitty_keyboard_flags" {
    var t: Terminal = null;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &t,
        .{
            .cols = 80,
            .rows = 24,
            .max_scrollback = 0,
        },
    ));
    defer free(t);

    var flags: u8 = undefined;
    try testing.expectEqual(Result.success, get(t, .kitty_keyboard_flags, @ptrCast(&flags)));
    try testing.expectEqual(0, flags);

    // Push kitty flags via VT sequence: CSI > 3 u (push disambiguate | report_events)
    vt_write(t, "\x1b[>3u", 5);

    try testing.expectEqual(Result.success, get(t, .kitty_keyboard_flags, @ptrCast(&flags)));
    try testing.expectEqual(3, flags);
}

test "get mouse_tracking" {
    var t: Terminal = null;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &t,
        .{
            .cols = 80,
            .rows = 24,
            .max_scrollback = 0,
        },
    ));
    defer free(t);

    var tracking: bool = undefined;
    try testing.expectEqual(Result.success, get(t, .mouse_tracking, @ptrCast(&tracking)));
    try testing.expect(!tracking);

    // Enable X10 mouse (DEC mode 9)
    const x10_mode: modes.ModeTag.Backing = @bitCast(modes.ModeTag{ .value = 9, .ansi = false });
    try testing.expectEqual(Result.success, mode_set(t, x10_mode, true));
    try testing.expectEqual(Result.success, get(t, .mouse_tracking, @ptrCast(&tracking)));
    try testing.expect(tracking);

    // Disable X10, enable normal mouse (DEC mode 1000)
    try testing.expectEqual(Result.success, mode_set(t, x10_mode, false));
    const normal_mode: modes.ModeTag.Backing = @bitCast(modes.ModeTag{ .value = 1000, .ansi = false });
    try testing.expectEqual(Result.success, mode_set(t, normal_mode, true));
    try testing.expectEqual(Result.success, get(t, .mouse_tracking, @ptrCast(&tracking)));
    try testing.expect(tracking);

    // Disable normal, enable button mouse (DEC mode 1002)
    try testing.expectEqual(Result.success, mode_set(t, normal_mode, false));
    const button_mode: modes.ModeTag.Backing = @bitCast(modes.ModeTag{ .value = 1002, .ansi = false });
    try testing.expectEqual(Result.success, mode_set(t, button_mode, true));
    try testing.expectEqual(Result.success, get(t, .mouse_tracking, @ptrCast(&tracking)));
    try testing.expect(tracking);

    // Disable button, enable any mouse (DEC mode 1003)
    try testing.expectEqual(Result.success, mode_set(t, button_mode, false));
    const any_mode: modes.ModeTag.Backing = @bitCast(modes.ModeTag{ .value = 1003, .ansi = false });
    try testing.expectEqual(Result.success, mode_set(t, any_mode, true));
    try testing.expectEqual(Result.success, get(t, .mouse_tracking, @ptrCast(&tracking)));
    try testing.expect(tracking);

    // Disable all - should be false again
    try testing.expectEqual(Result.success, mode_set(t, any_mode, false));
    try testing.expectEqual(Result.success, get(t, .mouse_tracking, @ptrCast(&tracking)));
    try testing.expect(!tracking);
}

test "get scrollbar" {
    var t: Terminal = null;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &t,
        .{
            .cols = 80,
            .rows = 3,
            .max_scrollback = 10_000,
        },
    ));
    defer free(t);

    var scrollbar: TerminalScrollbar = undefined;
    try testing.expectEqual(Result.success, get(t, .scrollbar, @ptrCast(&scrollbar)));
    try testing.expectEqual(@as(u64, 3), scrollbar.total);
    try testing.expectEqual(@as(u64, 0), scrollbar.offset);
    try testing.expectEqual(@as(u64, 3), scrollbar.len);

    vt_write(t, "line1\r\nline2\r\nline3\r\nline4\r\nline5\r\n", 34);

    try testing.expectEqual(Result.success, get(t, .scrollbar, @ptrCast(&scrollbar)));
    try testing.expectEqual(@as(u64, 5), scrollbar.total);
    try testing.expectEqual(@as(u64, 2), scrollbar.offset);
    try testing.expectEqual(@as(u64, 3), scrollbar.len);
}

test "get kitty image settings" {
    var t: Terminal = null;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &t,
        .{
            .cols = 80,
            .rows = 24,
            .max_scrollback = 0,
        },
    ));
    defer free(t);

    var limit: u64 = undefined;
    var medium_file: bool = undefined;
    var medium_temp_file: bool = undefined;
    var medium_shared_mem: bool = undefined;

    if (comptime !build_options.kitty_graphics) {
        try testing.expectEqual(Result.no_value, get(t, .kitty_image_storage_limit, @ptrCast(&limit)));
        try testing.expectEqual(Result.no_value, get(t, .kitty_image_medium_file, @ptrCast(&medium_file)));
        try testing.expectEqual(Result.no_value, get(t, .kitty_image_medium_temp_file, @ptrCast(&medium_temp_file)));
        try testing.expectEqual(Result.no_value, get(t, .kitty_image_medium_shared_mem, @ptrCast(&medium_shared_mem)));
        return;
    }

    try testing.expectEqual(Result.success, get(t, .kitty_image_storage_limit, @ptrCast(&limit)));
    if (comptime !build_options.terminal_rust_owned) {
        const zt = zigTerminalForTest(t) orelse return;
        try testing.expectEqual(@as(u64, @intCast(zt.screens.active.kitty_images.total_limit)), limit);
    }
    try testing.expectEqual(Result.success, get(t, .kitty_image_medium_file, @ptrCast(&medium_file)));
    try testing.expect(!medium_file);
    try testing.expectEqual(Result.success, get(t, .kitty_image_medium_temp_file, @ptrCast(&medium_temp_file)));
    try testing.expect(!medium_temp_file);
    try testing.expectEqual(Result.success, get(t, .kitty_image_medium_shared_mem, @ptrCast(&medium_shared_mem)));
    try testing.expect(!medium_shared_mem);

    const new_limit: u64 = 4096;
    const enabled = true;
    try testing.expectEqual(Result.success, set(t, .kitty_image_storage_limit, @ptrCast(&new_limit)));
    try testing.expectEqual(Result.success, set(t, .kitty_image_medium_file, @ptrCast(&enabled)));
    try testing.expectEqual(Result.success, set(t, .kitty_image_medium_temp_file, @ptrCast(&enabled)));
    try testing.expectEqual(Result.success, set(t, .kitty_image_medium_shared_mem, @ptrCast(&enabled)));

    try testing.expectEqual(Result.success, get(t, .kitty_image_storage_limit, @ptrCast(&limit)));
    try testing.expectEqual(new_limit, limit);
    try testing.expectEqual(Result.success, get(t, .kitty_image_medium_file, @ptrCast(&medium_file)));
    try testing.expect(medium_file);
    try testing.expectEqual(Result.success, get(t, .kitty_image_medium_temp_file, @ptrCast(&medium_temp_file)));
    try testing.expect(medium_temp_file);
    try testing.expectEqual(Result.success, get(t, .kitty_image_medium_shared_mem, @ptrCast(&medium_shared_mem)));
    try testing.expect(medium_shared_mem);
}

test "set APC max bytes" {
    var t: Terminal = null;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &t,
        .{
            .cols = 80,
            .rows = 24,
            .max_scrollback = 0,
        },
    ));
    defer free(t);

    if (comptime build_options.terminal_rust_owned) {
        const all: usize = 123;
        try testing.expectEqual(Result.success, set(t, .apc_max_bytes, @ptrCast(&all)));

        const kitty_max_bytes: usize = 456;
        try testing.expectEqual(Result.success, set(t, .apc_max_bytes_kitty, @ptrCast(&kitty_max_bytes)));

        try testing.expectEqual(Result.success, set(t, .apc_max_bytes_kitty, null));

        try testing.expectEqual(Result.success, set(t, .apc_max_bytes, @ptrCast(&all)));

        try testing.expectEqual(Result.success, set(t, .apc_max_bytes, null));
        return;
    }

    try testing.expectEqual(
        apc.Protocol.defaultMaxBytes(.kitty),
        t.?.stream.handler.apc_handler.max_bytes.get(.kitty).?,
    );

    const all: usize = 123;
    try testing.expectEqual(Result.success, set(t, .apc_max_bytes, @ptrCast(&all)));
    try testing.expectEqual(all, t.?.stream.handler.apc_handler.max_bytes.get(.kitty).?);

    const kitty_max_bytes: usize = 456;
    try testing.expectEqual(Result.success, set(t, .apc_max_bytes_kitty, @ptrCast(&kitty_max_bytes)));
    try testing.expectEqual(kitty_max_bytes, t.?.stream.handler.apc_handler.max_bytes.get(.kitty).?);

    try testing.expectEqual(Result.success, set(t, .apc_max_bytes_kitty, null));
    try testing.expectEqual(@as(?usize, null), t.?.stream.handler.apc_handler.max_bytes.get(.kitty));

    try testing.expectEqual(Result.success, set(t, .apc_max_bytes, @ptrCast(&all)));
    try testing.expectEqual(all, t.?.stream.handler.apc_handler.max_bytes.get(.kitty).?);

    try testing.expectEqual(Result.success, set(t, .apc_max_bytes, null));
    try testing.expectEqual(@as(?usize, null), t.?.stream.handler.apc_handler.max_bytes.get(.kitty));
}

test "get total_rows" {
    var t: Terminal = null;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &t,
        .{
            .cols = 80,
            .rows = 24,
            .max_scrollback = 10_000,
        },
    ));
    defer free(t);

    var total: usize = undefined;
    try testing.expectEqual(Result.success, get(t, .total_rows, @ptrCast(&total)));
    try testing.expect(total >= 24);
}

test "get scrollback_rows" {
    var t: Terminal = null;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &t,
        .{
            .cols = 80,
            .rows = 3,
            .max_scrollback = 10_000,
        },
    ));
    defer free(t);

    var scrollback: usize = undefined;
    try testing.expectEqual(Result.success, get(t, .scrollback_rows, @ptrCast(&scrollback)));
    try testing.expectEqual(@as(usize, 0), scrollback);

    // Write enough lines to push content into scrollback
    vt_write(t, "line1\r\nline2\r\nline3\r\nline4\r\nline5\r\n", 34);

    try testing.expectEqual(Result.success, get(t, .scrollback_rows, @ptrCast(&scrollback)));
    try testing.expectEqual(@as(usize, 2), scrollback);
}

test "get invalid" {
    var t: Terminal = null;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &t,
        .{
            .cols = 80,
            .rows = 24,
            .max_scrollback = 0,
        },
    ));
    defer free(t);

    try testing.expectEqual(Result.invalid_value, get(t, .invalid, null));
}

test "set and get selection" {
    var t: Terminal = null;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &t,
        .{
            .cols = 80,
            .rows = 24,
            .max_scrollback = 0,
        },
    ));
    defer free(t);

    vt_write(t, "Hello", 5);

    var start_ref: grid_ref_c.CGridRef = .{};
    try testing.expectEqual(Result.success, grid_ref(t, .{
        .tag = .active,
        .value = .{ .active = .{ .x = 0, .y = 0 } },
    }, &start_ref));

    var end_ref: grid_ref_c.CGridRef = .{};
    try testing.expectEqual(Result.success, grid_ref(t, .{
        .tag = .active,
        .value = .{ .active = .{ .x = 4, .y = 0 } },
    }, &end_ref));

    var out: selection_c.CSelection = undefined;
    try testing.expectEqual(Result.no_value, get(t, .selection, @ptrCast(&out)));

    const sel: selection_c.CSelection = .{
        .start = start_ref,
        .end = end_ref,
        .rectangle = true,
    };
    try testing.expectEqual(Result.success, set(t, .selection, @ptrCast(&sel)));
    if (comptime !build_options.terminal_rust_owned) {
        const zt = zigTerminalForTest(t) orelse return;
        try testing.expect(zt.screens.active.selection.?.tracked());
    }

    try testing.expectEqual(Result.success, get(t, .selection, @ptrCast(&out)));
    try testing.expect(out.start.toPin().?.eql(start_ref.toPin().?));
    try testing.expect(out.end.toPin().?.eql(end_ref.toPin().?));
    try testing.expect(out.rectangle);

    try testing.expectEqual(Result.success, set(t, .selection, null));
    if (comptime !build_options.terminal_rust_owned) {
        const zt = zigTerminalForTest(t) orelse return;
        try testing.expect(zt.screens.active.selection == null);
    }
    try testing.expectEqual(Result.no_value, get(t, .selection, @ptrCast(&out)));
}

test "selection derivation helpers" {
    if (comptime build_options.terminal_rust_owned) return;
    var t: Terminal = null;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &t,
        .{
            .cols = 80,
            .rows = 24,
            .max_scrollback = 0,
        },
    ));
    defer free(t);

    vt_write(t, "  Hello  \r\nWorld", 16);

    var out: selection_c.CSelection = undefined;

    var word_ref: grid_ref_c.CGridRef = .{};
    try testing.expectEqual(Result.success, grid_ref(t, .{
        .tag = .active,
        .value = .{ .active = .{ .x = 3, .y = 0 } },
    }, &word_ref));

    var empty_ref: grid_ref_c.CGridRef = .{};
    try testing.expectEqual(Result.success, grid_ref(t, .{
        .tag = .active,
        .value = .{ .active = .{ .x = 20, .y = 0 } },
    }, &empty_ref));

    var line_ref: grid_ref_c.CGridRef = .{};
    try testing.expectEqual(Result.success, grid_ref(t, .{
        .tag = .active,
        .value = .{ .active = .{ .x = 0, .y = 0 } },
    }, &line_ref));

    var word_opts: selection_c.SelectWordOptions = .{
        .ref = word_ref,
    };
    try testing.expectEqual(Result.success, selection_c.word(t, &word_opts, &out));
    try testing.expectEqual(@as(u16, 2), out.start.toPin().?.x);
    try testing.expectEqual(@as(u16, 6), out.end.toPin().?.x);

    word_opts.ref = empty_ref;
    try testing.expectEqual(Result.no_value, selection_c.word(t, &word_opts, &out));

    var between_start_ref: grid_ref_c.CGridRef = .{};
    try testing.expectEqual(Result.success, grid_ref(t, .{
        .tag = .active,
        .value = .{ .active = .{ .x = 20, .y = 1 } },
    }, &between_start_ref));

    var between_end_ref: grid_ref_c.CGridRef = .{};
    try testing.expectEqual(Result.success, grid_ref(t, .{
        .tag = .active,
        .value = .{ .active = .{ .x = 0, .y = 1 } },
    }, &between_end_ref));

    var word_between_opts: selection_c.SelectWordBetweenOptions = .{
        .start = between_start_ref,
        .end = between_end_ref,
    };
    try testing.expectEqual(Result.success, selection_c.word_between(t, &word_between_opts, &out));
    try testing.expectEqual(@as(u16, 0), out.start.toPin().?.x);
    try testing.expectEqual(@as(u16, 1), out.start.toPin().?.y);
    try testing.expectEqual(@as(u16, 4), out.end.toPin().?.x);
    try testing.expectEqual(@as(u16, 1), out.end.toPin().?.y);

    var line_opts: selection_c.SelectLineOptions = .{
        .ref = line_ref,
    };
    try testing.expectEqual(Result.success, selection_c.line(t, &line_opts, &out));
    try testing.expectEqual(@as(u16, 2), out.start.toPin().?.x);
    try testing.expectEqual(@as(u16, 6), out.end.toPin().?.x);

    try testing.expectEqual(Result.success, selection_c.all(t, &out));
    try testing.expectEqual(@as(u16, 2), out.start.toPin().?.x);
    try testing.expectEqual(@as(u16, 0), out.start.toPin().?.y);
    try testing.expectEqual(@as(u16, 4), out.end.toPin().?.x);
    try testing.expectEqual(@as(u16, 1), out.end.toPin().?.y);

    try testing.expectEqual(Result.no_value, selection_c.output(t, line_ref, &out));

    line_opts.size = @sizeOf(usize) - 1;
    try testing.expectEqual(Result.invalid_value, selection_c.line(t, &line_opts, &out));
    try testing.expectEqual(Result.invalid_value, selection_c.word(t, null, &out));
    try testing.expectEqual(Result.invalid_value, selection_c.word(t, &word_opts, null));
    try testing.expectEqual(Result.invalid_value, selection_c.word_between(t, null, &out));
    try testing.expectEqual(Result.invalid_value, selection_c.word_between(t, &word_between_opts, null));
}

test "selection_adjust mutates snapshot end" {
    if (comptime build_options.terminal_rust_owned) return;
    var t: Terminal = null;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &t,
        .{
            .cols = 80,
            .rows = 24,
            .max_scrollback = 0,
        },
    ));
    defer free(t);

    vt_write(t, "Hello", 5);

    var start_ref: grid_ref_c.CGridRef = .{};
    try testing.expectEqual(Result.success, grid_ref(t, .{
        .tag = .active,
        .value = .{ .active = .{ .x = 0, .y = 0 } },
    }, &start_ref));

    var end_ref: grid_ref_c.CGridRef = .{};
    try testing.expectEqual(Result.success, grid_ref(t, .{
        .tag = .active,
        .value = .{ .active = .{ .x = 1, .y = 0 } },
    }, &end_ref));

    var sel: selection_c.CSelection = .{
        .start = start_ref,
        .end = end_ref,
    };
    try testing.expectEqual(Result.success, selection_c.adjust(t, &sel, .right));
    try testing.expectEqual(@as(u16, 0), sel.start.toPin().?.x);
    try testing.expectEqual(@as(u16, 2), sel.end.toPin().?.x);

    try testing.expectEqual(Result.success, selection_c.adjust(t, &sel, .left));
    try testing.expectEqual(@as(u16, 0), sel.start.toPin().?.x);
    try testing.expectEqual(@as(u16, 1), sel.end.toPin().?.x);

    sel = .{
        .start = end_ref,
        .end = start_ref,
    };
    try testing.expectEqual(Result.success, selection_c.adjust(t, &sel, .right));
    try testing.expectEqual(@as(u16, 1), sel.start.toPin().?.x);
    try testing.expectEqual(@as(u16, 1), sel.end.toPin().?.x);
}

test "selection_order and selection_ordered" {
    if (comptime build_options.terminal_rust_owned) return;
    var t: Terminal = null;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &t,
        .{
            .cols = 80,
            .rows = 24,
            .max_scrollback = 0,
        },
    ));
    defer free(t);

    vt_write(t, "Hello\r\nWorld", 12);

    var start_ref: grid_ref_c.CGridRef = .{};
    try testing.expectEqual(Result.success, grid_ref(t, .{
        .tag = .active,
        .value = .{ .active = .{ .x = 3, .y = 0 } },
    }, &start_ref));

    var end_ref: grid_ref_c.CGridRef = .{};
    try testing.expectEqual(Result.success, grid_ref(t, .{
        .tag = .active,
        .value = .{ .active = .{ .x = 1, .y = 1 } },
    }, &end_ref));

    const sel: selection_c.CSelection = .{
        .start = start_ref,
        .end = end_ref,
        .rectangle = true,
    };

    var order: selection_c.Order = undefined;
    try testing.expectEqual(Result.success, selection_c.order(t, &sel, &order));
    try testing.expectEqual(selection_c.Order.mirrored_forward, order);

    var out: selection_c.CSelection = undefined;
    try testing.expectEqual(Result.success, selection_c.ordered(t, &sel, .forward, &out));
    try testing.expectEqual(@as(u16, 1), out.start.toPin().?.x);
    try testing.expectEqual(@as(u16, 0), out.start.toPin().?.y);
    try testing.expectEqual(@as(u16, 3), out.end.toPin().?.x);
    try testing.expectEqual(@as(u16, 1), out.end.toPin().?.y);
    try testing.expect(out.rectangle);

    try testing.expectEqual(Result.success, selection_c.ordered(t, &sel, .reverse, &out));
    try testing.expectEqual(@as(u16, 3), out.start.toPin().?.x);
    try testing.expectEqual(@as(u16, 1), out.start.toPin().?.y);
    try testing.expectEqual(@as(u16, 1), out.end.toPin().?.x);
    try testing.expectEqual(@as(u16, 0), out.end.toPin().?.y);
    try testing.expect(out.rectangle);
}

test "selection_contains" {
    if (comptime build_options.terminal_rust_owned) return;
    var t: Terminal = null;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &t,
        .{
            .cols = 80,
            .rows = 24,
            .max_scrollback = 0,
        },
    ));
    defer free(t);

    vt_write(t, "Hello\r\nWorld", 12);

    var start_ref: grid_ref_c.CGridRef = .{};
    try testing.expectEqual(Result.success, grid_ref(t, .{
        .tag = .active,
        .value = .{ .active = .{ .x = 3, .y = 0 } },
    }, &start_ref));

    var end_ref: grid_ref_c.CGridRef = .{};
    try testing.expectEqual(Result.success, grid_ref(t, .{
        .tag = .active,
        .value = .{ .active = .{ .x = 1, .y = 1 } },
    }, &end_ref));

    const linear: selection_c.CSelection = .{
        .start = start_ref,
        .end = end_ref,
    };

    var contains: bool = undefined;
    try testing.expectEqual(Result.success, selection_c.contains(t, &linear, .{
        .tag = .active,
        .value = .{ .active = .{ .x = 4, .y = 0 } },
    }, &contains));
    try testing.expect(contains);

    try testing.expectEqual(Result.success, selection_c.contains(t, &linear, .{
        .tag = .active,
        .value = .{ .active = .{ .x = 2, .y = 0 } },
    }, &contains));
    try testing.expect(!contains);

    const rectangle: selection_c.CSelection = .{
        .start = start_ref,
        .end = end_ref,
        .rectangle = true,
    };

    try testing.expectEqual(Result.success, selection_c.contains(t, &rectangle, .{
        .tag = .active,
        .value = .{ .active = .{ .x = 2, .y = 0 } },
    }, &contains));
    try testing.expect(contains);
}

test "selection_equal" {
    if (comptime build_options.terminal_rust_owned) return;
    var t: Terminal = null;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &t,
        .{
            .cols = 80,
            .rows = 24,
            .max_scrollback = 0,
        },
    ));
    defer free(t);

    var other_t: Terminal = null;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &other_t,
        .{
            .cols = 80,
            .rows = 24,
            .max_scrollback = 0,
        },
    ));
    defer free(other_t);

    vt_write(t, "Hello", 5);
    vt_write(other_t, "Hello", 5);

    var start_ref: grid_ref_c.CGridRef = .{};
    try testing.expectEqual(Result.success, grid_ref(t, .{
        .tag = .active,
        .value = .{ .active = .{ .x = 0, .y = 0 } },
    }, &start_ref));

    var end_ref: grid_ref_c.CGridRef = .{};
    try testing.expectEqual(Result.success, grid_ref(t, .{
        .tag = .active,
        .value = .{ .active = .{ .x = 1, .y = 0 } },
    }, &end_ref));

    var other_end_ref: grid_ref_c.CGridRef = .{};
    try testing.expectEqual(Result.success, grid_ref(t, .{
        .tag = .active,
        .value = .{ .active = .{ .x = 2, .y = 0 } },
    }, &other_end_ref));

    var cross_terminal_ref: grid_ref_c.CGridRef = .{};
    try testing.expectEqual(Result.success, grid_ref(other_t, .{
        .tag = .active,
        .value = .{ .active = .{ .x = 1, .y = 0 } },
    }, &cross_terminal_ref));

    const sel: selection_c.CSelection = .{
        .start = start_ref,
        .end = end_ref,
    };
    const equal_sel: selection_c.CSelection = .{
        .start = start_ref,
        .end = end_ref,
    };
    const different_endpoint: selection_c.CSelection = .{
        .start = start_ref,
        .end = other_end_ref,
    };
    const different_rectangle: selection_c.CSelection = .{
        .start = start_ref,
        .end = end_ref,
        .rectangle = true,
    };
    const cross_terminal: selection_c.CSelection = .{
        .start = start_ref,
        .end = cross_terminal_ref,
    };

    var equal: bool = undefined;
    try testing.expectEqual(Result.success, selection_c.equal(t, &sel, &equal_sel, &equal));
    try testing.expect(equal);

    try testing.expectEqual(Result.success, selection_c.equal(t, &sel, &different_endpoint, &equal));
    try testing.expect(!equal);

    try testing.expectEqual(Result.success, selection_c.equal(t, &sel, &different_rectangle, &equal));
    try testing.expect(!equal);

    try testing.expectEqual(Result.success, selection_c.equal(t, &sel, &cross_terminal, &equal));
    try testing.expect(!equal);
    try testing.expectEqual(Result.invalid_value, selection_c.equal(t, &sel, &equal_sel, null));
}

test "selection_order invalid values" {
    if (comptime build_options.terminal_rust_owned) return;
    var t: Terminal = null;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &t,
        .{
            .cols = 80,
            .rows = 24,
            .max_scrollback = 0,
        },
    ));
    defer free(t);

    var order: selection_c.Order = undefined;
    try testing.expectEqual(Result.invalid_value, selection_c.order(null, null, &order));
    try testing.expectEqual(Result.invalid_value, selection_c.order(t, null, &order));
}

test "grid_ref" {
    var t: Terminal = null;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &t,
        .{
            .cols = 80,
            .rows = 24,
            .max_scrollback = 0,
        },
    ));
    defer free(t);

    vt_write(t, "Hello", 5);

    var out_ref: grid_ref_c.CGridRef = .{};
    try testing.expectEqual(Result.success, grid_ref(t, .{
        .tag = .active,
        .value = .{ .active = .{ .x = 0, .y = 0 } },
    }, &out_ref));

    // Extract cell from grid ref and verify it contains 'H'
    var out_cell: cell_c.CCell = undefined;
    try testing.expectEqual(Result.success, grid_ref_c.grid_ref_cell(&out_ref, &out_cell));

    var cp: u32 = 0;
    try testing.expectEqual(Result.success, cell_c.get(out_cell, .codepoint, @ptrCast(&cp)));
    try testing.expectEqual(@as(u32, 'H'), cp);
}

test "grid_ref null terminal" {
    var out_ref: grid_ref_c.CGridRef = .{};
    try testing.expectEqual(Result.invalid_value, grid_ref(null, .{
        .tag = .active,
        .value = .{ .active = .{ .x = 0, .y = 0 } },
    }, &out_ref));
}

test "grid_ref null out succeeds" {
    var t: Terminal = null;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &t,
        .{
            .cols = 80,
            .rows = 24,
            .max_scrollback = 0,
        },
    ));
    defer free(t);

    vt_write(t, "Hello", 5);
    try testing.expectEqual(Result.success, grid_ref(t, .{
        .tag = .active,
        .value = .{ .active = .{ .x = 0, .y = 0 } },
    }, null));
}

test "grid_ref_track invalid inputs" {
    var out_ref: grid_ref_tracked_c.CTrackedGridRef = undefined;
    try testing.expectEqual(Result.invalid_value, grid_ref_track(null, .{
        .tag = .active,
        .value = .{ .active = .{ .x = 0, .y = 0 } },
    }, &out_ref));

    var t: Terminal = null;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &t,
        .{ .cols = 80, .rows = 24, .max_scrollback = 0 },
    ));
    defer free(t);

    try testing.expectEqual(Result.invalid_value, grid_ref_track(t, .{
        .tag = .active,
        .value = .{ .active = .{ .x = 0, .y = 0 } },
    }, null));
}

test "point_from_grid_ref roundtrip active" {
    var t: Terminal = null;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &t,
        .{ .cols = 80, .rows = 24, .max_scrollback = 10_000 },
    ));
    defer free(t);

    vt_write(t, "Hello", 5);

    // Get a grid ref at (2, 0) in active coords
    var ref: grid_ref_c.CGridRef = .{};
    try testing.expectEqual(Result.success, grid_ref(t, .{
        .tag = .active,
        .value = .{ .active = .{ .x = 2, .y = 0 } },
    }, &ref));

    // Convert back to active coords
    var coord: point.Coordinate = undefined;
    try testing.expectEqual(Result.success, point_from_grid_ref(t, &ref, .active, &coord));
    try testing.expectEqual(@as(size.CellCountInt, 2), coord.x);
    try testing.expectEqual(@as(u32, 0), coord.y);
}

test "point_from_grid_ref roundtrip viewport" {
    var t: Terminal = null;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &t,
        .{ .cols = 80, .rows = 24, .max_scrollback = 10_000 },
    ));
    defer free(t);

    vt_write(t, "Hello", 5);

    var ref: grid_ref_c.CGridRef = .{};
    try testing.expectEqual(Result.success, grid_ref(t, .{
        .tag = .viewport,
        .value = .{ .viewport = .{ .x = 0, .y = 0 } },
    }, &ref));

    var coord: point.Coordinate = undefined;
    try testing.expectEqual(Result.success, point_from_grid_ref(t, &ref, .viewport, &coord));
    try testing.expectEqual(@as(size.CellCountInt, 0), coord.x);
    try testing.expectEqual(@as(u32, 0), coord.y);
}

test "point_from_grid_ref null out succeeds" {
    var t: Terminal = null;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &t,
        .{ .cols = 80, .rows = 24, .max_scrollback = 10_000 },
    ));
    defer free(t);

    vt_write(t, "Hello", 5);

    var ref: grid_ref_c.CGridRef = .{};
    try testing.expectEqual(Result.success, grid_ref(t, .{
        .tag = .active,
        .value = .{ .active = .{ .x = 0, .y = 0 } },
    }, &ref));

    try testing.expectEqual(Result.success, point_from_grid_ref(t, &ref, .active, null));
}

test "point_from_grid_ref history ref to active returns no_value" {
    var t: Terminal = null;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &t,
        .{ .cols = 80, .rows = 4, .max_scrollback = 10_000 },
    ));
    defer free(t);

    // Write enough lines to push content into scrollback
    for (0..10) |_| {
        vt_write(t, "line\n", 5);
    }

    // Get a ref to the first line (now in scrollback)
    var ref: grid_ref_c.CGridRef = .{};
    try testing.expectEqual(Result.success, grid_ref(t, .{
        .tag = .screen,
        .value = .{ .screen = .{ .x = 0, .y = 0 } },
    }, &ref));

    // Should succeed for screen coords
    var coord: point.Coordinate = undefined;
    try testing.expectEqual(Result.success, point_from_grid_ref(t, &ref, .screen, &coord));
    try testing.expectEqual(@as(u32, 0), coord.y);

    // Should fail for active coords (it's in scrollback)
    try testing.expectEqual(Result.no_value, point_from_grid_ref(t, &ref, .active, &coord));
}

test "point_from_grid_ref null terminal" {
    var ref: grid_ref_c.CGridRef = .{};
    try testing.expectEqual(Result.invalid_value, point_from_grid_ref(null, &ref, .active, null));
}

test "point_from_grid_ref null ref" {
    var t: Terminal = null;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &t,
        .{ .cols = 80, .rows = 24, .max_scrollback = 0 },
    ));
    defer free(t);

    try testing.expectEqual(Result.invalid_value, point_from_grid_ref(t, null, .active, null));
}

test "point_from_grid_ref null node" {
    var t: Terminal = null;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &t,
        .{ .cols = 80, .rows = 24, .max_scrollback = 0 },
    ));
    defer free(t);

    const ref: grid_ref_c.CGridRef = .{};
    try testing.expectEqual(Result.invalid_value, point_from_grid_ref(t, &ref, .active, null));
}

test "set write_pty callback" {
    var t: Terminal = null;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &t,
        .{
            .cols = 80,
            .rows = 24,
            .max_scrollback = 0,
        },
    ));
    defer free(t);

    const S = struct {
        var last_data: ?[]u8 = null;
        var last_userdata: ?*anyopaque = null;

        fn deinit() void {
            if (last_data) |d| testing.allocator.free(d);
            last_data = null;
            last_userdata = null;
        }

        fn writePty(_: Terminal, ud: ?*anyopaque, ptr: [*]const u8, len: usize) callconv(lib.calling_conv) void {
            if (last_data) |d| testing.allocator.free(d);
            last_data = testing.allocator.dupe(u8, ptr[0..len]) catch @panic("OOM");
            last_userdata = ud;
        }
    };
    defer S.deinit();

    // Set userdata and write_pty callback
    var sentinel: u8 = 42;
    try testing.expectEqual(Result.success, set(t, .userdata, @ptrCast(&sentinel)));
    try testing.expectEqual(Result.success, set(t, .write_pty, @ptrCast(&S.writePty)));

    // DECRQM for wraparound mode (mode 7, set by default) should trigger write_pty
    vt_write(t, "\x1B[?7$p", 6);
    try testing.expect(S.last_data != null);
    try testing.expectEqualStrings("\x1B[?7;1$y", S.last_data.?);
    try testing.expectEqual(@as(?*anyopaque, @ptrCast(&sentinel)), S.last_userdata);
}

test "set write_pty without callback ignores queries" {
    var t: Terminal = null;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &t,
        .{
            .cols = 80,
            .rows = 24,
            .max_scrollback = 0,
        },
    ));
    defer free(t);

    // Without setting a callback, DECRQM should be silently ignored (no crash)
    vt_write(t, "\x1B[?7$p", 6);
}

test "set write_pty null clears callback" {
    var t: Terminal = null;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &t,
        .{
            .cols = 80,
            .rows = 24,
            .max_scrollback = 0,
        },
    ));
    defer free(t);

    const S = struct {
        var called: bool = false;
        fn writePty(_: Terminal, _: ?*anyopaque, _: [*]const u8, _: usize) callconv(lib.calling_conv) void {
            called = true;
        }
    };
    S.called = false;

    // Set then clear the callback
    try testing.expectEqual(Result.success, set(t, .write_pty, @ptrCast(&S.writePty)));
    try testing.expectEqual(Result.success, set(t, .write_pty, null));

    vt_write(t, "\x1B[?7$p", 6);
    try testing.expect(!S.called);
}

test "set bell callback" {
    var t: Terminal = null;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &t,
        .{
            .cols = 80,
            .rows = 24,
            .max_scrollback = 0,
        },
    ));
    defer free(t);

    const S = struct {
        var bell_count: usize = 0;
        var last_userdata: ?*anyopaque = null;

        fn bell(_: Terminal, ud: ?*anyopaque) callconv(lib.calling_conv) void {
            bell_count += 1;
            last_userdata = ud;
        }
    };
    S.bell_count = 0;
    S.last_userdata = null;

    // Set userdata and bell callback
    var sentinel: u8 = 99;
    try testing.expectEqual(Result.success, set(t, .userdata, @ptrCast(&sentinel)));
    try testing.expectEqual(Result.success, set(t, .bell, @ptrCast(&S.bell)));

    // Single BEL
    vt_write(t, "\x07", 1);
    try testing.expectEqual(@as(usize, 1), S.bell_count);
    try testing.expectEqual(@as(?*anyopaque, @ptrCast(&sentinel)), S.last_userdata);

    // Multiple BELs
    vt_write(t, "\x07\x07", 2);
    try testing.expectEqual(@as(usize, 3), S.bell_count);
}

test "bell without callback is silent" {
    var t: Terminal = null;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &t,
        .{
            .cols = 80,
            .rows = 24,
            .max_scrollback = 0,
        },
    ));
    defer free(t);

    // BEL without a callback should not crash
    vt_write(t, "\x07", 1);
}

test "set enquiry callback" {
    var t: Terminal = null;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &t,
        .{
            .cols = 80,
            .rows = 24,
            .max_scrollback = 0,
        },
    ));
    defer free(t);

    const S = struct {
        var last_data: ?[]u8 = null;

        fn deinit() void {
            if (last_data) |d| testing.allocator.free(d);
            last_data = null;
        }

        fn writePty(_: Terminal, _: ?*anyopaque, ptr: [*]const u8, len: usize) callconv(lib.calling_conv) void {
            if (last_data) |d| testing.allocator.free(d);
            last_data = testing.allocator.dupe(u8, ptr[0..len]) catch @panic("OOM");
        }

        const response = "OK";
        fn enquiry(_: Terminal, _: ?*anyopaque) callconv(lib.calling_conv) lib.String {
            return .{ .ptr = response, .len = response.len };
        }
    };
    defer S.deinit();

    try testing.expectEqual(Result.success, set(t, .write_pty, @ptrCast(&S.writePty)));
    try testing.expectEqual(Result.success, set(t, .enquiry, @ptrCast(&S.enquiry)));

    // ENQ (0x05) should trigger the enquiry callback and write response via write_pty
    vt_write(t, "\x05", 1);
    try testing.expect(S.last_data != null);
    try testing.expectEqualStrings("OK", S.last_data.?);
}

test "enquiry without callback is silent" {
    var t: Terminal = null;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &t,
        .{
            .cols = 80,
            .rows = 24,
            .max_scrollback = 0,
        },
    ));
    defer free(t);

    // ENQ without a callback should not crash
    vt_write(t, "\x05", 1);
}

test "set xtversion callback" {
    var t: Terminal = null;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &t,
        .{
            .cols = 80,
            .rows = 24,
            .max_scrollback = 0,
        },
    ));
    defer free(t);

    const S = struct {
        var last_data: ?[]u8 = null;

        fn deinit() void {
            if (last_data) |d| testing.allocator.free(d);
            last_data = null;
        }

        fn writePty(_: Terminal, _: ?*anyopaque, ptr: [*]const u8, len: usize) callconv(lib.calling_conv) void {
            if (last_data) |d| testing.allocator.free(d);
            last_data = testing.allocator.dupe(u8, ptr[0..len]) catch @panic("OOM");
        }

        const version = "myterm 1.0";
        fn xtversion(_: Terminal, _: ?*anyopaque) callconv(lib.calling_conv) lib.String {
            return .{ .ptr = version, .len = version.len };
        }
    };
    defer S.deinit();

    try testing.expectEqual(Result.success, set(t, .write_pty, @ptrCast(&S.writePty)));
    try testing.expectEqual(Result.success, set(t, .xtversion, @ptrCast(&S.xtversion)));

    // XTVERSION: CSI > q
    vt_write(t, "\x1B[>q", 4);
    try testing.expect(S.last_data != null);
    // Response should be DCS >| version ST
    try testing.expectEqualStrings("\x1BP>|myterm 1.0\x1B\\", S.last_data.?);
}

test "xtversion without callback reports default" {
    var t: Terminal = null;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &t,
        .{
            .cols = 80,
            .rows = 24,
            .max_scrollback = 0,
        },
    ));
    defer free(t);

    const S = struct {
        var last_data: ?[]u8 = null;

        fn deinit() void {
            if (last_data) |d| testing.allocator.free(d);
            last_data = null;
        }

        fn writePty(_: Terminal, _: ?*anyopaque, ptr: [*]const u8, len: usize) callconv(lib.calling_conv) void {
            if (last_data) |d| testing.allocator.free(d);
            last_data = testing.allocator.dupe(u8, ptr[0..len]) catch @panic("OOM");
        }
    };
    defer S.deinit();

    // Set write_pty but not xtversion — should get default "libghostty"
    try testing.expectEqual(Result.success, set(t, .write_pty, @ptrCast(&S.writePty)));

    vt_write(t, "\x1B[>q", 4);
    try testing.expect(S.last_data != null);
    try testing.expectEqualStrings("\x1BP>|libghostty\x1B\\", S.last_data.?);
}

test "set title_changed callback" {
    var t: Terminal = null;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &t,
        .{
            .cols = 80,
            .rows = 24,
            .max_scrollback = 0,
        },
    ));
    defer free(t);

    const S = struct {
        var title_count: usize = 0;
        var last_userdata: ?*anyopaque = null;

        fn titleChanged(_: Terminal, ud: ?*anyopaque) callconv(lib.calling_conv) void {
            title_count += 1;
            last_userdata = ud;
        }
    };
    S.title_count = 0;
    S.last_userdata = null;

    var sentinel: u8 = 77;
    try testing.expectEqual(Result.success, set(t, .userdata, @ptrCast(&sentinel)));
    try testing.expectEqual(Result.success, set(t, .title_changed, @ptrCast(&S.titleChanged)));

    // OSC 2 ; title ST — set window title
    vt_write(t, "\x1B]2;Hello\x1B\\", 10);
    try testing.expectEqual(@as(usize, 1), S.title_count);
    try testing.expectEqual(@as(?*anyopaque, @ptrCast(&sentinel)), S.last_userdata);

    // Another title change
    vt_write(t, "\x1B]2;World\x1B\\", 10);
    try testing.expectEqual(@as(usize, 2), S.title_count);
}

test "title_changed without callback is silent" {
    var t: Terminal = null;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &t,
        .{
            .cols = 80,
            .rows = 24,
            .max_scrollback = 0,
        },
    ));
    defer free(t);

    // OSC 2 without a callback should not crash
    vt_write(t, "\x1B]2;Hello\x1B\\", 10);
}

test "set size callback" {
    var t: Terminal = null;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &t,
        .{
            .cols = 80,
            .rows = 24,
            .max_scrollback = 0,
        },
    ));
    defer free(t);

    const S = struct {
        var last_data: ?[]u8 = null;

        fn deinit() void {
            if (last_data) |d| testing.allocator.free(d);
            last_data = null;
        }

        fn writePty(_: Terminal, _: ?*anyopaque, ptr: [*]const u8, len: usize) callconv(lib.calling_conv) void {
            if (last_data) |d| testing.allocator.free(d);
            last_data = testing.allocator.dupe(u8, ptr[0..len]) catch @panic("OOM");
        }

        fn sizeCb(_: Terminal, _: ?*anyopaque, out_size: *size_report.Size) callconv(lib.calling_conv) bool {
            out_size.* = .{
                .rows = 24,
                .columns = 80,
                .cell_width = 8,
                .cell_height = 16,
            };
            return true;
        }
    };
    defer S.deinit();

    try testing.expectEqual(Result.success, set(t, .write_pty, @ptrCast(&S.writePty)));
    try testing.expectEqual(Result.success, set(t, .size_cb, @ptrCast(&S.sizeCb)));

    // CSI 18 t — report text area size in characters
    vt_write(t, "\x1B[18t", 5);
    try testing.expect(S.last_data != null);
    try testing.expectEqualStrings("\x1b[8;24;80t", S.last_data.?);
}

test "size without callback is silent" {
    var t: Terminal = null;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &t,
        .{
            .cols = 80,
            .rows = 24,
            .max_scrollback = 0,
        },
    ));
    defer free(t);

    // CSI 18 t without a size callback should not crash
    vt_write(t, "\x1B[18t", 5);
}

test "set device_attributes callback primary" {
    var t: Terminal = null;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &t,
        .{
            .cols = 80,
            .rows = 24,
            .max_scrollback = 0,
        },
    ));
    defer free(t);

    const S = struct {
        var last_data: ?[]u8 = null;

        fn deinit() void {
            if (last_data) |d| testing.allocator.free(d);
            last_data = null;
        }

        fn writePty(_: Terminal, _: ?*anyopaque, ptr: [*]const u8, len: usize) callconv(lib.calling_conv) void {
            if (last_data) |d| testing.allocator.free(d);
            last_data = testing.allocator.dupe(u8, ptr[0..len]) catch @panic("OOM");
        }

        fn da(_: Terminal, _: ?*anyopaque, out: *Effects.CDeviceAttributes) callconv(lib.calling_conv) bool {
            out.* = .{
                .primary = .{
                    .conformance_level = 64,
                    .features = .{ 22, 52 } ++ .{0} ** 62,
                    .num_features = 2,
                },
                .secondary = .{
                    .device_type = 1,
                    .firmware_version = 10,
                    .rom_cartridge = 0,
                },
                .tertiary = .{ .unit_id = 0 },
            };
            return true;
        }
    };
    defer S.deinit();

    try testing.expectEqual(Result.success, set(t, .write_pty, @ptrCast(&S.writePty)));
    try testing.expectEqual(Result.success, set(t, .device_attributes, @ptrCast(&S.da)));

    // CSI c — primary DA
    vt_write(t, "\x1B[c", 3);
    try testing.expect(S.last_data != null);
    try testing.expectEqualStrings("\x1b[?64;22;52c", S.last_data.?);
}

test "set device_attributes callback secondary" {
    var t: Terminal = null;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &t,
        .{
            .cols = 80,
            .rows = 24,
            .max_scrollback = 0,
        },
    ));
    defer free(t);

    const S = struct {
        var last_data: ?[]u8 = null;

        fn deinit() void {
            if (last_data) |d| testing.allocator.free(d);
            last_data = null;
        }

        fn writePty(_: Terminal, _: ?*anyopaque, ptr: [*]const u8, len: usize) callconv(lib.calling_conv) void {
            if (last_data) |d| testing.allocator.free(d);
            last_data = testing.allocator.dupe(u8, ptr[0..len]) catch @panic("OOM");
        }

        fn da(_: Terminal, _: ?*anyopaque, out: *Effects.CDeviceAttributes) callconv(lib.calling_conv) bool {
            out.* = .{
                .primary = .{
                    .conformance_level = 62,
                    .features = .{22} ++ .{0} ** 63,
                    .num_features = 1,
                },
                .secondary = .{
                    .device_type = 1,
                    .firmware_version = 10,
                    .rom_cartridge = 0,
                },
                .tertiary = .{ .unit_id = 0 },
            };
            return true;
        }
    };
    defer S.deinit();

    try testing.expectEqual(Result.success, set(t, .write_pty, @ptrCast(&S.writePty)));
    try testing.expectEqual(Result.success, set(t, .device_attributes, @ptrCast(&S.da)));

    // CSI > c — secondary DA
    vt_write(t, "\x1B[>c", 4);
    try testing.expect(S.last_data != null);
    try testing.expectEqualStrings("\x1b[>1;10;0c", S.last_data.?);
}

test "set device_attributes callback tertiary" {
    var t: Terminal = null;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &t,
        .{
            .cols = 80,
            .rows = 24,
            .max_scrollback = 0,
        },
    ));
    defer free(t);

    const S = struct {
        var last_data: ?[]u8 = null;

        fn deinit() void {
            if (last_data) |d| testing.allocator.free(d);
            last_data = null;
        }

        fn writePty(_: Terminal, _: ?*anyopaque, ptr: [*]const u8, len: usize) callconv(lib.calling_conv) void {
            if (last_data) |d| testing.allocator.free(d);
            last_data = testing.allocator.dupe(u8, ptr[0..len]) catch @panic("OOM");
        }

        fn da(_: Terminal, _: ?*anyopaque, out: *Effects.CDeviceAttributes) callconv(lib.calling_conv) bool {
            out.* = .{
                .primary = .{
                    .conformance_level = 62,
                    .features = .{0} ** 64,
                    .num_features = 0,
                },
                .secondary = .{
                    .device_type = 1,
                    .firmware_version = 0,
                    .rom_cartridge = 0,
                },
                .tertiary = .{ .unit_id = 0xAABBCCDD },
            };
            return true;
        }
    };
    defer S.deinit();

    try testing.expectEqual(Result.success, set(t, .write_pty, @ptrCast(&S.writePty)));
    try testing.expectEqual(Result.success, set(t, .device_attributes, @ptrCast(&S.da)));

    // CSI = c — tertiary DA
    vt_write(t, "\x1B[=c", 4);
    try testing.expect(S.last_data != null);
    try testing.expectEqualStrings("\x1bP!|AABBCCDD\x1b\\", S.last_data.?);
}

test "device_attributes without callback uses default" {
    var t: Terminal = null;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &t,
        .{
            .cols = 80,
            .rows = 24,
            .max_scrollback = 0,
        },
    ));
    defer free(t);

    const S = struct {
        var last_data: ?[]u8 = null;

        fn deinit() void {
            if (last_data) |d| testing.allocator.free(d);
            last_data = null;
        }

        fn writePty(_: Terminal, _: ?*anyopaque, ptr: [*]const u8, len: usize) callconv(lib.calling_conv) void {
            if (last_data) |d| testing.allocator.free(d);
            last_data = testing.allocator.dupe(u8, ptr[0..len]) catch @panic("OOM");
        }
    };
    defer S.deinit();

    try testing.expectEqual(Result.success, set(t, .write_pty, @ptrCast(&S.writePty)));

    // Without setting a device_attributes callback, DA1 should return the default
    vt_write(t, "\x1B[c", 3);
    try testing.expect(S.last_data != null);
    try testing.expectEqualStrings("\x1b[?62;22c", S.last_data.?);
}

test "device_attributes callback returns false uses default" {
    var t: Terminal = null;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &t,
        .{
            .cols = 80,
            .rows = 24,
            .max_scrollback = 0,
        },
    ));
    defer free(t);

    const S = struct {
        var last_data: ?[]u8 = null;

        fn deinit() void {
            if (last_data) |d| testing.allocator.free(d);
            last_data = null;
        }

        fn writePty(_: Terminal, _: ?*anyopaque, ptr: [*]const u8, len: usize) callconv(lib.calling_conv) void {
            if (last_data) |d| testing.allocator.free(d);
            last_data = testing.allocator.dupe(u8, ptr[0..len]) catch @panic("OOM");
        }

        fn da(_: Terminal, _: ?*anyopaque, _: *Effects.CDeviceAttributes) callconv(lib.calling_conv) bool {
            return false;
        }
    };
    defer S.deinit();

    try testing.expectEqual(Result.success, set(t, .write_pty, @ptrCast(&S.writePty)));
    try testing.expectEqual(Result.success, set(t, .device_attributes, @ptrCast(&S.da)));

    // Callback returns false, should use default response
    vt_write(t, "\x1B[c", 3);
    try testing.expect(S.last_data != null);
    try testing.expectEqualStrings("\x1b[?62;22c", S.last_data.?);
}

test "set and get title" {
    var t: Terminal = null;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &t,
        .{
            .cols = 80,
            .rows = 24,
            .max_scrollback = 0,
        },
    ));
    defer free(t);

    // No title set yet — should return empty string
    var title: lib.String = undefined;
    try testing.expectEqual(Result.success, get(t, .title, @ptrCast(&title)));
    try testing.expectEqual(@as(usize, 0), title.len);

    // Set title via option
    const hello: lib.String = .{ .ptr = "Hello", .len = 5 };
    try testing.expectEqual(Result.success, set(t, .title, @ptrCast(&hello)));

    try testing.expectEqual(Result.success, get(t, .title, @ptrCast(&title)));
    try testing.expectEqualStrings("Hello", title.ptr[0..title.len]);

    // Overwrite title
    const world: lib.String = .{ .ptr = "World", .len = 5 };
    try testing.expectEqual(Result.success, set(t, .title, @ptrCast(&world)));

    try testing.expectEqual(Result.success, get(t, .title, @ptrCast(&title)));
    try testing.expectEqualStrings("World", title.ptr[0..title.len]);

    // Clear title with NULL
    try testing.expectEqual(Result.success, set(t, .title, null));

    try testing.expectEqual(Result.success, get(t, .title, @ptrCast(&title)));
    try testing.expectEqual(@as(usize, 0), title.len);
}

test "set and get pwd" {
    var t: Terminal = null;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &t,
        .{
            .cols = 80,
            .rows = 24,
            .max_scrollback = 0,
        },
    ));
    defer free(t);

    // No pwd set yet — should return empty string
    var pwd: lib.String = undefined;
    try testing.expectEqual(Result.success, get(t, .pwd, @ptrCast(&pwd)));
    try testing.expectEqual(@as(usize, 0), pwd.len);

    // Set pwd via option
    const home: lib.String = .{ .ptr = "/home/user", .len = 10 };
    try testing.expectEqual(Result.success, set(t, .pwd, @ptrCast(&home)));

    try testing.expectEqual(Result.success, get(t, .pwd, @ptrCast(&pwd)));
    try testing.expectEqualStrings("/home/user", pwd.ptr[0..pwd.len]);

    // Clear pwd with NULL
    try testing.expectEqual(Result.success, set(t, .pwd, null));

    try testing.expectEqual(Result.success, get(t, .pwd, @ptrCast(&pwd)));
    try testing.expectEqual(@as(usize, 0), pwd.len);
}

test "get title set via vt_write" {
    var t: Terminal = null;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &t,
        .{
            .cols = 80,
            .rows = 24,
            .max_scrollback = 0,
        },
    ));
    defer free(t);

    // Set title via OSC 2
    vt_write(t, "\x1B]2;VT Title\x1B\\", 14);

    var title: lib.String = undefined;
    try testing.expectEqual(Result.success, get(t, .title, @ptrCast(&title)));
    try testing.expectEqualStrings("VT Title", title.ptr[0..title.len]);
}

test "resize updates pixel dimensions" {
    var t: Terminal = null;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &t,
        .{
            .cols = 80,
            .rows = 24,
            .max_scrollback = 0,
        },
    ));
    defer free(t);

    try testing.expectEqual(Result.success, resize(t, 100, 40, 9, 18));

    var width_px: u32 = undefined;
    var height_px: u32 = undefined;
    try testing.expectEqual(Result.success, get(t, .width_px, @ptrCast(&width_px)));
    try testing.expectEqual(Result.success, get(t, .height_px, @ptrCast(&height_px)));
    try testing.expectEqual(@as(u32, 100 * 9), width_px);
    try testing.expectEqual(@as(u32, 40 * 18), height_px);
}

test "resize pixel overflow saturates" {
    var t: Terminal = null;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &t,
        .{
            .cols = 80,
            .rows = 24,
            .max_scrollback = 0,
        },
    ));
    defer free(t);

    try testing.expectEqual(Result.success, resize(t, 100, 40, std.math.maxInt(u32), std.math.maxInt(u32)));

    var width_px: u32 = undefined;
    var height_px: u32 = undefined;
    try testing.expectEqual(Result.success, get(t, .width_px, @ptrCast(&width_px)));
    try testing.expectEqual(Result.success, get(t, .height_px, @ptrCast(&height_px)));
    try testing.expectEqual(std.math.maxInt(u32), width_px);
    try testing.expectEqual(std.math.maxInt(u32), height_px);
}

test "resize disables synchronized output" {
    var t: Terminal = null;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &t,
        .{
            .cols = 80,
            .rows = 24,
            .max_scrollback = 0,
        },
    ));
    defer free(t);

    const synchronized_output: modes.ModeTag.Backing = @bitCast(modes.ModeTag{
        .value = 2026,
        .ansi = false,
    });
    try testing.expectEqual(Result.success, mode_set(t, synchronized_output, true));

    try testing.expectEqual(Result.success, resize(t, 100, 40, 9, 18));

    var value: bool = undefined;
    try testing.expectEqual(Result.success, mode_get(t, synchronized_output, &value));
    try testing.expect(!value);
}

test "resize sends in-band size report" {
    var t: Terminal = null;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &t,
        .{
            .cols = 80,
            .rows = 24,
            .max_scrollback = 0,
        },
    ));
    defer free(t);

    const S = struct {
        var last_data: ?[]u8 = null;

        fn deinit() void {
            if (last_data) |d| testing.allocator.free(d);
            last_data = null;
        }

        fn writePty(_: Terminal, _: ?*anyopaque, ptr: [*]const u8, len: usize) callconv(lib.calling_conv) void {
            if (last_data) |d| testing.allocator.free(d);
            last_data = testing.allocator.dupe(u8, ptr[0..len]) catch @panic("OOM");
        }
    };
    defer S.deinit();

    try testing.expectEqual(Result.success, set(t, .write_pty, @ptrCast(&S.writePty)));

    const in_band_size_reports: modes.ModeTag.Backing = @bitCast(modes.ModeTag{
        .value = 2048,
        .ansi = false,
    });
    try testing.expectEqual(Result.success, mode_set(t, in_band_size_reports, true));

    try testing.expectEqual(Result.success, resize(t, 100, 40, 9, 18));

    // Expected: \x1B[48;rows;cols;height_px;width_pxt
    // height_px = 40*18 = 720, width_px = 100*9 = 900
    try testing.expect(S.last_data != null);
    try testing.expectEqualStrings("\x1B[48;40;100;720;900t", S.last_data.?);
}

test "resize no size report without mode 2048" {
    var t: Terminal = null;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &t,
        .{
            .cols = 80,
            .rows = 24,
            .max_scrollback = 0,
        },
    ));
    defer free(t);

    const S = struct {
        var called: bool = false;
        fn writePty(_: Terminal, _: ?*anyopaque, _: [*]const u8, _: usize) callconv(lib.calling_conv) void {
            called = true;
        }
    };
    S.called = false;

    try testing.expectEqual(Result.success, set(t, .write_pty, @ptrCast(&S.writePty)));

    // in_band_size_reports is off by default
    try testing.expectEqual(Result.success, resize(t, 100, 40, 9, 18));
    try testing.expect(!S.called);
}

test "resize in-band report without write_pty callback" {
    var t: Terminal = null;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &t,
        .{
            .cols = 80,
            .rows = 24,
            .max_scrollback = 0,
        },
    ));
    defer free(t);

    const in_band_size_reports: modes.ModeTag.Backing = @bitCast(modes.ModeTag{
        .value = 2048,
        .ansi = false,
    });
    try testing.expectEqual(Result.success, mode_set(t, in_band_size_reports, true));
    try testing.expectEqual(Result.success, resize(t, 100, 40, 9, 18));
}

test "resize null terminal" {
    try testing.expectEqual(Result.invalid_value, resize(null, 100, 40, 9, 18));
}

test "resize zero cols" {
    var t: Terminal = null;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &t,
        .{
            .cols = 80,
            .rows = 24,
            .max_scrollback = 0,
        },
    ));
    defer free(t);

    try testing.expectEqual(Result.invalid_value, resize(t, 0, 40, 9, 18));
}

test "resize zero rows" {
    var t: Terminal = null;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &t,
        .{
            .cols = 80,
            .rows = 24,
            .max_scrollback = 0,
        },
    ));
    defer free(t);

    try testing.expectEqual(Result.invalid_value, resize(t, 100, 0, 9, 18));
}

test "grid_ref out of bounds" {
    var t: Terminal = null;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &t,
        .{
            .cols = 80,
            .rows = 24,
            .max_scrollback = 0,
        },
    ));
    defer free(t);

    var out_ref: grid_ref_c.CGridRef = .{};
    try testing.expectEqual(Result.invalid_value, grid_ref(t, .{
        .tag = .active,
        .value = .{ .active = .{ .x = 100, .y = 0 } },
    }, &out_ref));
}

test "set and get color_foreground" {
    var t: Terminal = null;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &t,
        .{
            .cols = 80,
            .rows = 24,
            .max_scrollback = 0,
        },
    ));
    defer free(t);

    // Initially unset
    var rgb: color.RGB.C = undefined;
    try testing.expectEqual(Result.no_value, get(t, .color_foreground, @ptrCast(&rgb)));

    // Set a value
    const fg: color.RGB.C = .{ .r = 0xAA, .g = 0xBB, .b = 0xCC };
    try testing.expectEqual(Result.success, set(t, .color_foreground, @ptrCast(&fg)));
    try testing.expectEqual(Result.success, get(t, .color_foreground, @ptrCast(&rgb)));
    try testing.expectEqual(fg, rgb);

    // Clear with null
    try testing.expectEqual(Result.success, set(t, .color_foreground, null));
    try testing.expectEqual(Result.no_value, get(t, .color_foreground, @ptrCast(&rgb)));
}

test "set and get color_background" {
    var t: Terminal = null;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &t,
        .{
            .cols = 80,
            .rows = 24,
            .max_scrollback = 0,
        },
    ));
    defer free(t);

    var rgb: color.RGB.C = undefined;
    try testing.expectEqual(Result.no_value, get(t, .color_background, @ptrCast(&rgb)));

    const bg: color.RGB.C = .{ .r = 0x11, .g = 0x22, .b = 0x33 };
    try testing.expectEqual(Result.success, set(t, .color_background, @ptrCast(&bg)));
    try testing.expectEqual(Result.success, get(t, .color_background, @ptrCast(&rgb)));
    try testing.expectEqual(bg, rgb);

    try testing.expectEqual(Result.success, set(t, .color_background, null));
    try testing.expectEqual(Result.no_value, get(t, .color_background, @ptrCast(&rgb)));
}

test "set and get color_cursor" {
    var t: Terminal = null;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &t,
        .{
            .cols = 80,
            .rows = 24,
            .max_scrollback = 0,
        },
    ));
    defer free(t);

    var rgb: color.RGB.C = undefined;
    try testing.expectEqual(Result.no_value, get(t, .color_cursor, @ptrCast(&rgb)));

    const cur: color.RGB.C = .{ .r = 0xFF, .g = 0x00, .b = 0x88 };
    try testing.expectEqual(Result.success, set(t, .color_cursor, @ptrCast(&cur)));
    try testing.expectEqual(Result.success, get(t, .color_cursor, @ptrCast(&rgb)));
    try testing.expectEqual(cur, rgb);

    try testing.expectEqual(Result.success, set(t, .color_cursor, null));
    try testing.expectEqual(Result.no_value, get(t, .color_cursor, @ptrCast(&rgb)));
}

test "set and get color_palette" {
    var t: Terminal = null;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &t,
        .{
            .cols = 80,
            .rows = 24,
            .max_scrollback = 0,
        },
    ));
    defer free(t);

    // Get default palette
    var palette: color.PaletteC = undefined;
    try testing.expectEqual(Result.success, get(t, .color_palette, @ptrCast(&palette)));
    try testing.expectEqual(color.default[0].cval(), palette[0]);

    // Set custom palette
    var custom: color.PaletteC = color.paletteCval(&color.default);
    custom[0] = .{ .r = 0x12, .g = 0x34, .b = 0x56 };
    try testing.expectEqual(Result.success, set(t, .color_palette, @ptrCast(&custom)));
    try testing.expectEqual(Result.success, get(t, .color_palette, @ptrCast(&palette)));
    try testing.expectEqual(custom[0], palette[0]);

    // Reset with null restores default
    try testing.expectEqual(Result.success, set(t, .color_palette, null));
    try testing.expectEqual(Result.success, get(t, .color_palette, @ptrCast(&palette)));
    try testing.expectEqual(color.default[0].cval(), palette[0]);
}

test "get color default vs effective with override" {
    var t: Terminal = null;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &t,
        .{
            .cols = 80,
            .rows = 24,
            .max_scrollback = 0,
        },
    ));
    defer free(t);

    var rgb: color.RGB.C = undefined;

    // Set defaults
    const fg: color.RGB.C = .{ .r = 0xAA, .g = 0xBB, .b = 0xCC };
    const bg: color.RGB.C = .{ .r = 0x11, .g = 0x22, .b = 0x33 };
    const cur: color.RGB.C = .{ .r = 0xFF, .g = 0x00, .b = 0x88 };
    try testing.expectEqual(Result.success, set(t, .color_foreground, @ptrCast(&fg)));
    try testing.expectEqual(Result.success, set(t, .color_background, @ptrCast(&bg)));
    try testing.expectEqual(Result.success, set(t, .color_cursor, @ptrCast(&cur)));

    // Simulate OSC overrides
    const override: color.RGB.C = .{ .r = 0x00, .g = 0x00, .b = 0x00 };
    if (comptime build_options.terminal_rust_owned) {
        const handle = rustOwnedHandle(t.?) orelse return;
        try testing.expectEqual(Result.success, @as(Result, @enumFromInt(rust_owned.ghostty_rust_terminal_owned_set_color_override(
            handle,
            @intFromEnum(TerminalData.color_foreground),
            @ptrCast(&override),
        ))));
        try testing.expectEqual(Result.success, @as(Result, @enumFromInt(rust_owned.ghostty_rust_terminal_owned_set_color_override(
            handle,
            @intFromEnum(TerminalData.color_background),
            @ptrCast(&override),
        ))));
        try testing.expectEqual(Result.success, @as(Result, @enumFromInt(rust_owned.ghostty_rust_terminal_owned_set_color_override(
            handle,
            @intFromEnum(TerminalData.color_cursor),
            @ptrCast(&override),
        ))));
    } else {
        const zt = zigTerminalForTest(t) orelse return;
        const override_zig: color.RGB = .{ .r = 0x00, .g = 0x00, .b = 0x00 };
        zt.colors.foreground.override = override_zig;
        zt.colors.background.override = override_zig;
        zt.colors.cursor.override = override_zig;
    }

    // Effective returns override
    try testing.expectEqual(Result.success, get(t, .color_foreground, @ptrCast(&rgb)));
    try testing.expectEqual(override, rgb);
    try testing.expectEqual(Result.success, get(t, .color_background, @ptrCast(&rgb)));
    try testing.expectEqual(override, rgb);
    try testing.expectEqual(Result.success, get(t, .color_cursor, @ptrCast(&rgb)));
    try testing.expectEqual(override, rgb);

    // Default returns original
    try testing.expectEqual(Result.success, get(t, .color_foreground_default, @ptrCast(&rgb)));
    try testing.expectEqual(fg, rgb);
    try testing.expectEqual(Result.success, get(t, .color_background_default, @ptrCast(&rgb)));
    try testing.expectEqual(bg, rgb);
    try testing.expectEqual(Result.success, get(t, .color_cursor_default, @ptrCast(&rgb)));
    try testing.expectEqual(cur, rgb);
}

test "get color default returns no_value when unset" {
    var t: Terminal = null;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &t,
        .{
            .cols = 80,
            .rows = 24,
            .max_scrollback = 0,
        },
    ));
    defer free(t);

    var rgb: color.RGB.C = undefined;
    try testing.expectEqual(Result.no_value, get(t, .color_foreground_default, @ptrCast(&rgb)));
    try testing.expectEqual(Result.no_value, get(t, .color_background_default, @ptrCast(&rgb)));
    try testing.expectEqual(Result.no_value, get(t, .color_cursor_default, @ptrCast(&rgb)));
}

test "get color_palette_default vs current" {
    var t: Terminal = null;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &t,
        .{
            .cols = 80,
            .rows = 24,
            .max_scrollback = 0,
        },
    ));
    defer free(t);

    // Set a custom default palette
    var custom: color.PaletteC = color.paletteCval(&color.default);
    custom[0] = .{ .r = 0x12, .g = 0x34, .b = 0x56 };
    try testing.expectEqual(Result.success, set(t, .color_palette, @ptrCast(&custom)));

    // Simulate OSC override on index 0
    const override_entry: color.RGB.C = .{ .r = 0xFF, .g = 0xFF, .b = 0xFF };
    if (comptime build_options.terminal_rust_owned) {
        const handle = rustOwnedHandle(t.?) orelse return;
        try testing.expectEqual(Result.success, @as(Result, @enumFromInt(rust_owned.ghostty_rust_terminal_owned_set_palette_index(
            handle,
            0,
            @ptrCast(&override_entry),
        ))));
    } else {
        const zt = zigTerminalForTest(t) orelse return;
        zt.colors.palette.set(0, .{ .r = 0xFF, .g = 0xFF, .b = 0xFF });
    }

    // Current palette returns the override
    var palette: color.PaletteC = undefined;
    try testing.expectEqual(Result.success, get(t, .color_palette, @ptrCast(&palette)));
    try testing.expectEqual(color.RGB.C{ .r = 0xFF, .g = 0xFF, .b = 0xFF }, palette[0]);

    // Default palette returns the original
    try testing.expectEqual(Result.success, get(t, .color_palette_default, @ptrCast(&palette)));
    try testing.expectEqual(custom[0], palette[0]);
}

test "set color sets dirty flag" {
    var t: Terminal = null;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &t,
        .{
            .cols = 80,
            .rows = 24,
            .max_scrollback = 0,
        },
    ));
    defer free(t);

    const zt = zigTerminalForTest(t) orelse return;
    zt.flags.dirty.palette = false;

    const fg: color.RGB.C = .{ .r = 0xFF, .g = 0xFF, .b = 0xFF };
    try testing.expectEqual(Result.success, set(t, .color_foreground, @ptrCast(&fg)));
    try testing.expect(zt.flags.dirty.palette);
}

test "get_multi success" {
    var t: Terminal = null;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &t,
        .{ .cols = 80, .rows = 24, .max_scrollback = 0 },
    ));
    defer free(t);

    var cols: u16 = 0;
    var rows: u16 = 0;
    var written: usize = 0;

    const keys = [_]TerminalData{ .cols, .rows };
    var values = [_]?*anyopaque{ @ptrCast(&cols), @ptrCast(&rows) };
    try testing.expectEqual(Result.success, get_multi(t, keys.len, &keys, &values, &written));
    try testing.expectEqual(keys.len, written);
    try testing.expectEqual(80, cols);
    try testing.expectEqual(24, rows);
}

test "get_multi error sets out_written" {
    var t: Terminal = null;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &t,
        .{ .cols = 80, .rows = 24, .max_scrollback = 0 },
    ));
    defer free(t);

    var cols: u16 = 0;
    var written: usize = 99;

    const keys = [_]TerminalData{ .cols, .invalid };
    var values = [_]?*anyopaque{ @ptrCast(&cols), @ptrCast(&cols) };
    try testing.expectEqual(Result.invalid_value, get_multi(t, keys.len, &keys, &values, &written));
    try testing.expectEqual(1, written);
    try testing.expectEqual(80, cols);
}

test "get_multi null keys returns invalid_value" {
    var cols: u16 = 0;
    var values = [_]?*anyopaque{@ptrCast(&cols)};
    try testing.expectEqual(Result.invalid_value, get_multi(null, 1, null, &values, null));
}
