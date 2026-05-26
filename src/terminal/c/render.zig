const std = @import("std");
const testing = std.testing;
const build_options = @import("terminal_options");
const Allocator = std.mem.Allocator;
const lib = @import("../lib.zig");
const CAllocator = lib.alloc.Allocator;
const colorpkg = @import("../color.zig");
const cursorpkg = @import("../cursor.zig");
const page = @import("../page.zig");
const size = @import("../size.zig");
const Style = @import("../style.zig").Style;
const terminal_c = @import("terminal.zig");
const ZigTerminal = @import("../Terminal.zig");
const renderpkg = @import("../render.zig");
const Result = @import("result.zig").Result;
const row = @import("row.zig");
const style_c = @import("style.zig");

const log = std.log.scoped(.render_state_c);

const rust = if (build_options.lib_vt_rust) struct {
    extern fn ghostty_rust_render_index_next(
        has_current: bool,
        current: size.CellCountInt,
        len: usize,
        out_next: *size.CellCountInt,
    ) callconv(.c) bool;

    extern fn ghostty_rust_render_index_select(
        index: size.CellCountInt,
        len: usize,
    ) callconv(.c) c_int;

    extern fn ghostty_rust_render_row_cell_selected(
        selection_present: bool,
        selection_start: size.CellCountInt,
        selection_end: size.CellCountInt,
        x: size.CellCountInt,
        out: *bool,
    ) callconv(.c) c_int;

    extern fn ghostty_rust_render_state_get_primitive(
        data: c_int,
        cols: size.CellCountInt,
        rows: size.CellCountInt,
        dirty: c_int,
        cursor_visual_style: c_int,
        cursor_visible: bool,
        cursor_blinking: bool,
        cursor_password_input: bool,
        cursor_viewport_has_value: bool,
        cursor_viewport_x: size.CellCountInt,
        cursor_viewport_y: size.CellCountInt,
        cursor_viewport_wide_tail: bool,
        out: *anyopaque,
    ) callconv(.c) c_int;

    extern fn ghostty_rust_render_state_get_color(
        data: c_int,
        background: colorpkg.RGB.C,
        foreground: colorpkg.RGB.C,
        cursor_present: bool,
        cursor: colorpkg.RGB.C,
        palette: *const colorpkg.PaletteC,
        out: *anyopaque,
    ) callconv(.c) c_int;

    extern fn ghostty_rust_render_state_get_multi(
        count: usize,
        keys: ?[*]const Data,
        values: ?[*]?*anyopaque,
        out_written: ?*usize,
        cols: size.CellCountInt,
        rows: size.CellCountInt,
        dirty: c_int,
        cursor_visual_style: c_int,
        cursor_visible: bool,
        cursor_blinking: bool,
        cursor_password_input: bool,
        cursor_viewport_has_value: bool,
        cursor_viewport_x: size.CellCountInt,
        cursor_viewport_y: size.CellCountInt,
        cursor_viewport_wide_tail: bool,
        background: colorpkg.RGB.C,
        foreground: colorpkg.RGB.C,
        cursor_present: bool,
        cursor: colorpkg.RGB.C,
        palette: *const colorpkg.PaletteC,
    ) callconv(.c) c_int;

    extern fn ghostty_rust_render_state_set_dirty(
        value: Dirty,
        out: *Dirty,
    ) callconv(.c) c_int;

    extern fn ghostty_rust_render_state_set(
        has_state: bool,
        option: c_int,
        has_value: bool,
    ) callconv(.c) c_int;

    extern fn ghostty_rust_render_state_colors_get(
        out_size: usize,
        out: *Colors,
        background: colorpkg.RGB.C,
        foreground: colorpkg.RGB.C,
        cursor_present: bool,
        cursor: colorpkg.RGB.C,
        palette: *const colorpkg.PaletteC,
    ) callconv(.c) c_int;

    extern fn ghostty_rust_render_row_get_dirty(
        dirty: bool,
        out: *bool,
    ) callconv(.c) c_int;

    extern fn ghostty_rust_render_row_get_data(
        data: c_int,
        raw: row.CRow,
        dirty: bool,
        selection_present: bool,
        selection_start: size.CellCountInt,
        selection_end: size.CellCountInt,
        out_size: usize,
        out: *anyopaque,
    ) callconv(.c) c_int;

    extern fn ghostty_rust_render_row_get_selection(
        selection_present: bool,
        selection_start: size.CellCountInt,
        selection_end: size.CellCountInt,
        out_size: usize,
        out: *RowSelection,
    ) callconv(.c) c_int;

    extern fn ghostty_rust_render_row_get(
        has_iterator: bool,
        has_row: bool,
        data: c_int,
        has_out: bool,
        out_size: usize,
    ) callconv(.c) c_int;

    extern fn ghostty_rust_render_row_get_multi(
        count: usize,
        keys: ?[*]const RowData,
        values: ?[*]?*anyopaque,
        out_written: ?*usize,
        raw: row.CRow,
        dirty: bool,
        selection_present: bool,
        selection_start: size.CellCountInt,
        selection_end: size.CellCountInt,
    ) callconv(.c) c_int;

    extern fn ghostty_rust_render_row_set_dirty(
        value: bool,
        out: *bool,
    ) callconv(.c) c_int;

    extern fn ghostty_rust_render_row_set(
        has_iterator: bool,
        has_row: bool,
        option: c_int,
        has_value: bool,
    ) callconv(.c) c_int;

    extern fn ghostty_rust_render_row_cell_get_text(
        data: c_int,
        cell: page.Cell.C,
        extra: ?[*]const u21,
        extra_len: usize,
        out: *anyopaque,
    ) callconv(.c) c_int;

    extern fn ghostty_rust_render_row_cell_get(
        has_cells: bool,
        has_cell: bool,
        data: c_int,
        has_out: bool,
    ) callconv(.c) c_int;

    extern fn ghostty_rust_render_row_cell_get_multi(
        count: usize,
        keys: ?[*]const RowCellsData,
        values: ?[*]?*anyopaque,
        out_written: ?*usize,
        cell: page.Cell.C,
        extra: ?[*]const u21,
        extra_len: usize,
        fg_color: *const style_c.Color,
        bg_color: *const style_c.Color,
        underline_color: *const style_c.Color,
        bold: bool,
        italic: bool,
        faint: bool,
        blink: bool,
        inverse: bool,
        invisible: bool,
        strikethrough: bool,
        overline: bool,
        underline: c_int,
        cell_palette_color: colorpkg.RGB.C,
        fg_palette_color: colorpkg.RGB.C,
        bg_palette_color: colorpkg.RGB.C,
        selection_present: bool,
        selection_start: size.CellCountInt,
        selection_end: size.CellCountInt,
        x: size.CellCountInt,
    ) callconv(.c) c_int;

    extern fn ghostty_rust_render_row_cell_get_color(
        data: c_int,
        cell: page.Cell.C,
        fg_color: *const style_c.Color,
        bg_color: *const style_c.Color,
        cell_palette_color: colorpkg.RGB.C,
        fg_palette_color: colorpkg.RGB.C,
        bg_palette_color: colorpkg.RGB.C,
        out: *anyopaque,
    ) callconv(.c) c_int;

    extern fn ghostty_rust_render_row_cell_get_style(
        fg_color: *const style_c.Color,
        bg_color: *const style_c.Color,
        underline_color: *const style_c.Color,
        bold: bool,
        italic: bool,
        faint: bool,
        blink: bool,
        inverse: bool,
        invisible: bool,
        strikethrough: bool,
        overline: bool,
        underline: c_int,
        out: *style_c.Style,
    ) callconv(.c) c_int;
} else struct {};

const RenderStateWrapper = struct {
    alloc: std.mem.Allocator,
    state: renderpkg.RenderState = .empty,
};

const RowIteratorWrapper = struct {
    alloc: std.mem.Allocator,

    /// The current index (also y value) into the row list.
    y: ?size.CellCountInt,

    /// These are the raw pointers into the render state data.
    raws: []const page.Row,
    cells: []const std.MultiArrayList(renderpkg.RenderState.Cell),
    selection: []const ?[2]size.CellCountInt,
    dirty: []bool,

    /// The color palette from the render state, needed to resolve
    /// palette-indexed background colors on cells.
    palette: *const colorpkg.Palette,
};

const RowCellsWrapper = struct {
    alloc: std.mem.Allocator,
    x: ?size.CellCountInt,
    raws: []const page.Cell,
    graphemes: []const []const u21,
    styles: []const Style,
    selection: ?[2]size.CellCountInt,

    /// The color palette, needed to resolve palette-indexed background colors.
    palette: *const colorpkg.Palette,
};

/// C: GhosttyRenderState
pub const RenderState = ?*RenderStateWrapper;

/// C: GhosttyRenderStateRowIterator
pub const RowIterator = ?*RowIteratorWrapper;

/// C: GhosttyRenderStateRowCells
pub const RowCells = ?*RowCellsWrapper;

/// C: GhosttyRenderStateDirty
pub const Dirty = renderpkg.RenderState.Dirty;

/// C: GhosttyRenderStateRowSelection
pub const RowSelection = extern struct {
    size: usize = @sizeOf(RowSelection),
    start_x: u16 = 0,
    end_x: u16 = 0,
};

/// C: GhosttyRenderStateCursorVisualStyle
pub const CursorVisualStyle = enum(c_int) {
    bar = 0,
    block = 1,
    underline = 2,
    block_hollow = 3,

    pub fn fromCursorStyle(s: cursorpkg.Style) CursorVisualStyle {
        return switch (s) {
            .bar => .bar,
            .block => .block,
            .underline => .underline,
            .block_hollow => .block_hollow,
        };
    }
};

/// C: GhosttyRenderStateData
pub const Data = enum(c_int) {
    invalid = 0,
    cols = 1,
    rows = 2,
    dirty = 3,
    row_iterator = 4,
    color_background = 5,
    color_foreground = 6,
    color_cursor = 7,
    color_cursor_has_value = 8,
    color_palette = 9,
    cursor_visual_style = 10,
    cursor_visible = 11,
    cursor_blinking = 12,
    cursor_password_input = 13,
    cursor_viewport_has_value = 14,
    cursor_viewport_x = 15,
    cursor_viewport_y = 16,
    cursor_viewport_wide_tail = 17,

    /// Output type expected for querying the data of the given kind.
    pub fn OutType(comptime self: Data) type {
        return switch (self) {
            .invalid => void,
            .cols, .rows => size.CellCountInt,
            .dirty => Dirty,
            .row_iterator => RowIterator,
            .color_background, .color_foreground, .color_cursor => colorpkg.RGB.C,
            .color_cursor_has_value => bool,
            .color_palette => colorpkg.PaletteC,
            .cursor_visual_style => CursorVisualStyle,
            .cursor_visible, .cursor_blinking, .cursor_password_input => bool,
            .cursor_viewport_has_value, .cursor_viewport_wide_tail => bool,
            .cursor_viewport_x, .cursor_viewport_y => size.CellCountInt,
        };
    }
};

/// C: GhosttyRenderStateOption
pub const SetOption = enum(c_int) {
    dirty = 0,

    /// Input type expected for setting the option.
    pub fn InType(comptime self: SetOption) type {
        return switch (self) {
            .dirty => Dirty,
        };
    }
};

/// C: GhosttyRenderStateColors
pub const Colors = extern struct {
    size: usize = @sizeOf(Colors),
    background: colorpkg.RGB.C,
    foreground: colorpkg.RGB.C,
    cursor: colorpkg.RGB.C,
    cursor_has_value: bool,
    palette: colorpkg.PaletteC,
};

pub fn new(
    alloc_: ?*const CAllocator,
    result: *RenderState,
) callconv(lib.calling_conv) Result {
    result.* = new_(alloc_) catch |err| {
        result.* = null;
        return switch (err) {
            error.OutOfMemory => .out_of_memory,
        };
    };

    return .success;
}

fn new_(alloc_: ?*const CAllocator) error{OutOfMemory}!*RenderStateWrapper {
    const alloc = lib.alloc.default(alloc_);
    const ptr = alloc.create(RenderStateWrapper) catch
        return error.OutOfMemory;
    ptr.* = .{ .alloc = alloc };
    return ptr;
}

pub fn free(state_: RenderState) callconv(lib.calling_conv) void {
    const state = state_ orelse return;
    const alloc = state.alloc;
    state.state.deinit(alloc);
    alloc.destroy(state);
}

pub fn update(
    state_: RenderState,
    terminal_: terminal_c.Terminal,
) callconv(lib.calling_conv) Result {
    const state = state_ orelse return .invalid_value;
    const t: *ZigTerminal = (terminal_ orelse return .invalid_value).terminal;

    state.state.update(state.alloc, t) catch return .out_of_memory;
    return .success;
}

pub fn get(
    state_: RenderState,
    data: Data,
    out: ?*anyopaque,
) callconv(lib.calling_conv) Result {
    if (comptime std.debug.runtime_safety) {
        _ = std.meta.intToEnum(Data, @intFromEnum(data)) catch {
            log.warn("render_state_get invalid data value={d}", .{@intFromEnum(data)});
            return .invalid_value;
        };
    }

    return switch (data) {
        .invalid => .invalid_value,
        inline else => |comptime_data| getTyped(
            state_,
            comptime_data,
            @ptrCast(@alignCast(out)),
        ),
    };
}

pub fn get_multi(
    state_: RenderState,
    count: usize,
    keys: ?[*]const Data,
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
        var needs_zig = false;
        for (0..count) |i| {
            if (k[i] == .row_iterator) {
                needs_zig = true;
                break;
            }
        }

        if (!needs_zig) {
            const state = state_ orelse return .invalid_value;
            const viewport = state.state.cursor.viewport;
            const colors = state.state.colors;
            const cursor = if (colors.cursor) |cursor_value| cursor_value.cval() else colorpkg.RGB.C{
                .r = 0,
                .g = 0,
                .b = 0,
            };
            const palette = colorpkg.paletteCval(&colors.palette);

            return @enumFromInt(rust.ghostty_rust_render_state_get_multi(
                count,
                k,
                v,
                out_written,
                state.state.cols,
                state.state.rows,
                @intFromEnum(state.state.dirty),
                @intFromEnum(state.state.cursor.visual_style),
                state.state.cursor.visible,
                state.state.cursor.blinking,
                state.state.cursor.password_input,
                viewport != null,
                if (viewport) |vp| vp.x else 0,
                if (viewport) |vp| vp.y else 0,
                if (viewport) |vp| vp.wide_tail else false,
                colors.background.cval(),
                colors.foreground.cval(),
                colors.cursor != null,
                cursor,
                &palette,
            ));
        }
    }

    for (0..count) |i| {
        const result = get(state_, k[i], v[i]);
        if (result != .success) {
            if (out_written) |w| w.* = i;
            return result;
        }
    }
    if (out_written) |w| w.* = count;
    return .success;
}

fn getTyped(
    state_: RenderState,
    comptime data: Data,
    out: *data.OutType(),
) Result {
    const state = state_ orelse return .invalid_value;

    if (comptime build_options.lib_vt_rust) {
        switch (data) {
            .cols,
            .rows,
            .dirty,
            .cursor_visual_style,
            .cursor_visible,
            .cursor_blinking,
            .cursor_password_input,
            .cursor_viewport_has_value,
            .cursor_viewport_x,
            .cursor_viewport_y,
            .cursor_viewport_wide_tail,
            => {
                const viewport = state.state.cursor.viewport;
                return @enumFromInt(rust.ghostty_rust_render_state_get_primitive(
                    @intFromEnum(data),
                    state.state.cols,
                    state.state.rows,
                    @intFromEnum(state.state.dirty),
                    @intFromEnum(state.state.cursor.visual_style),
                    state.state.cursor.visible,
                    state.state.cursor.blinking,
                    state.state.cursor.password_input,
                    viewport != null,
                    if (viewport) |vp| vp.x else 0,
                    if (viewport) |vp| vp.y else 0,
                    if (viewport) |vp| vp.wide_tail else false,
                    @ptrCast(out),
                ));
            },
            .color_background,
            .color_foreground,
            .color_cursor,
            .color_cursor_has_value,
            .color_palette,
            => {
                const colors = state.state.colors;
                const cursor = if (colors.cursor) |cursor_value| cursor_value.cval() else colorpkg.RGB.C{
                    .r = 0,
                    .g = 0,
                    .b = 0,
                };
                const palette = colorpkg.paletteCval(&colors.palette);
                return @enumFromInt(rust.ghostty_rust_render_state_get_color(
                    @intFromEnum(data),
                    colors.background.cval(),
                    colors.foreground.cval(),
                    colors.cursor != null,
                    cursor,
                    &palette,
                    @ptrCast(out),
                ));
            },
            else => {},
        }
    }

    switch (data) {
        .invalid => return .invalid_value,
        .cols => out.* = state.state.cols,
        .rows => out.* = state.state.rows,
        .dirty => out.* = state.state.dirty,
        .row_iterator => {
            const it = out.* orelse return .invalid_value;
            const row_data = state.state.row_data.slice();
            it.* = .{
                .alloc = it.alloc,
                .y = null,
                .raws = row_data.items(.raw),
                .cells = row_data.items(.cells),
                .selection = row_data.items(.selection),
                .dirty = row_data.items(.dirty),
                .palette = &state.state.colors.palette,
            };
        },
        .color_background => out.* = state.state.colors.background.cval(),
        .color_foreground => out.* = state.state.colors.foreground.cval(),
        .color_cursor => {
            const cursor = state.state.colors.cursor orelse return .invalid_value;
            out.* = cursor.cval();
        },
        .color_cursor_has_value => out.* = state.state.colors.cursor != null,
        .color_palette => out.* = colorpkg.paletteCval(&state.state.colors.palette),
        .cursor_visual_style => out.* = CursorVisualStyle.fromCursorStyle(state.state.cursor.visual_style),
        .cursor_visible => out.* = state.state.cursor.visible,
        .cursor_blinking => out.* = state.state.cursor.blinking,
        .cursor_password_input => out.* = state.state.cursor.password_input,
        .cursor_viewport_has_value => out.* = state.state.cursor.viewport != null,
        .cursor_viewport_x => {
            const vp = state.state.cursor.viewport orelse return .invalid_value;
            out.* = vp.x;
        },
        .cursor_viewport_y => {
            const vp = state.state.cursor.viewport orelse return .invalid_value;
            out.* = vp.y;
        },
        .cursor_viewport_wide_tail => {
            const vp = state.state.cursor.viewport orelse return .invalid_value;
            out.* = vp.wide_tail;
        },
    }

    return .success;
}

pub fn set(
    state_: RenderState,
    option: SetOption,
    value: ?*const anyopaque,
) callconv(lib.calling_conv) Result {
    if (comptime build_options.lib_vt_rust) {
        const result: Result = @enumFromInt(rust.ghostty_rust_render_state_set(
            state_ != null,
            @intFromEnum(option),
            value != null,
        ));
        if (result != .success) return result;
    } else if (comptime std.debug.runtime_safety) {
        _ = std.meta.intToEnum(SetOption, @intFromEnum(option)) catch {
            log.warn("render_state_set invalid option value={d}", .{@intFromEnum(option)});
            return .invalid_value;
        };
    }

    return switch (option) {
        inline else => |comptime_option| setTyped(
            state_,
            comptime_option,
            @ptrCast(@alignCast(value orelse return .invalid_value)),
        ),
    };
}

fn setTyped(
    state_: RenderState,
    comptime option: SetOption,
    value: *const option.InType(),
) Result {
    const state = state_ orelse return .invalid_value;
    switch (option) {
        .dirty => {
            if (comptime build_options.lib_vt_rust) {
                return @enumFromInt(rust.ghostty_rust_render_state_set_dirty(
                    value.*,
                    &state.state.dirty,
                ));
            }

            state.state.dirty = value.*;
        },
    }

    return .success;
}

pub fn colors_get(
    state_: RenderState,
    out_colors_: ?*Colors,
) callconv(lib.calling_conv) Result {
    const state = state_ orelse return .invalid_value;
    const out_colors = out_colors_ orelse return .invalid_value;
    const out_size = out_colors.size;
    if (out_size < @sizeOf(usize)) return .invalid_value;

    const colors = state.state.colors;
    if (comptime build_options.lib_vt_rust) {
        const cursor = if (colors.cursor) |cursor| cursor.cval() else colorpkg.RGB.C{
            .r = 0,
            .g = 0,
            .b = 0,
        };
        const palette = colorpkg.paletteCval(&colors.palette);
        return @enumFromInt(rust.ghostty_rust_render_state_colors_get(
            out_size,
            out_colors,
            colors.background.cval(),
            colors.foreground.cval(),
            colors.cursor != null,
            cursor,
            &palette,
        ));
    }

    if (lib.structSizedFieldFits(
        Colors,
        out_size,
        "background",
    )) {
        out_colors.background = colors.background.cval();
    }

    if (lib.structSizedFieldFits(
        Colors,
        out_size,
        "foreground",
    )) {
        out_colors.foreground = colors.foreground.cval();
    }

    if (colors.cursor) |cursor| {
        if (lib.structSizedFieldFits(
            Colors,
            out_size,
            "cursor",
        )) {
            out_colors.cursor = cursor.cval();
        }
    }

    if (lib.structSizedFieldFits(
        Colors,
        out_size,
        "cursor_has_value",
    )) {
        out_colors.cursor_has_value = colors.cursor != null;
    }

    {
        const palette_offset = @offsetOf(Colors, "palette");
        if (out_size > palette_offset) {
            const available = out_size - palette_offset;
            const max_entries = @min(colors.palette.len, available / @sizeOf(colorpkg.RGB.C));
            for (0..max_entries) |i| {
                out_colors.palette[i] = colors.palette[i].cval();
            }
        }
    }

    return .success;
}

pub fn row_iterator_new(
    alloc_: ?*const CAllocator,
    result: *RowIterator,
) callconv(lib.calling_conv) Result {
    const alloc = lib.alloc.default(alloc_);
    const ptr = alloc.create(RowIteratorWrapper) catch {
        result.* = null;
        return .out_of_memory;
    };
    ptr.* = .{
        .alloc = alloc,
        .y = undefined,
        .raws = undefined,
        .cells = undefined,
        .selection = undefined,
        .dirty = undefined,
        .palette = undefined,
    };
    result.* = ptr;
    return .success;
}

pub fn row_iterator_free(iterator_: RowIterator) callconv(lib.calling_conv) void {
    const iterator = iterator_ orelse return;
    const alloc = iterator.alloc;
    alloc.destroy(iterator);
}

pub fn row_iterator_next(iterator_: RowIterator) callconv(lib.calling_conv) bool {
    const it = iterator_ orelse return false;
    if (comptime build_options.lib_vt_rust) {
        var next_y: size.CellCountInt = undefined;
        if (!rust.ghostty_rust_render_index_next(
            it.y != null,
            it.y orelse 0,
            it.raws.len,
            &next_y,
        )) return false;
        it.y = next_y;
        return true;
    }

    const next_y: size.CellCountInt = if (it.y) |y| y + 1 else 0;
    if (next_y >= it.raws.len) return false;
    it.y = next_y;
    return true;
}

pub fn row_cells_new(
    alloc_: ?*const CAllocator,
    result: *RowCells,
) callconv(lib.calling_conv) Result {
    const alloc = lib.alloc.default(alloc_);
    const ptr = alloc.create(RowCellsWrapper) catch {
        result.* = null;
        return .out_of_memory;
    };
    ptr.* = .{
        .alloc = alloc,
        .x = undefined,
        .raws = undefined,
        .graphemes = undefined,
        .styles = undefined,
        .selection = undefined,
        .palette = undefined,
    };
    result.* = ptr;
    return .success;
}

pub fn row_cells_next(cells_: RowCells) callconv(lib.calling_conv) bool {
    const cells = cells_ orelse return false;
    if (comptime build_options.lib_vt_rust) {
        var next_x: size.CellCountInt = undefined;
        if (!rust.ghostty_rust_render_index_next(
            cells.x != null,
            cells.x orelse 0,
            cells.raws.len,
            &next_x,
        )) return false;
        cells.x = next_x;
        return true;
    }

    const next_x: size.CellCountInt = if (cells.x) |x| x + 1 else 0;
    if (next_x >= cells.raws.len) return false;
    cells.x = next_x;
    return true;
}

pub fn row_cells_select(cells_: RowCells, x: size.CellCountInt) callconv(lib.calling_conv) Result {
    const cells = cells_ orelse return .invalid_value;
    if (comptime build_options.lib_vt_rust) {
        const result: Result = @enumFromInt(rust.ghostty_rust_render_index_select(
            x,
            cells.raws.len,
        ));
        if (result != .success) return result;
        cells.x = x;
        return .success;
    }

    if (x >= cells.raws.len) return .invalid_value;
    cells.x = x;
    return .success;
}

pub fn row_cells_free(cells_: RowCells) callconv(lib.calling_conv) void {
    const cells = cells_ orelse return;
    const alloc = cells.alloc;
    alloc.destroy(cells);
}

/// C: GhosttyRenderStateRowCellsData
pub const RowCellsData = enum(c_int) {
    invalid = 0,
    raw = 1,
    style = 2,
    graphemes_len = 3,
    graphemes_buf = 4,
    bg_color = 5,
    fg_color = 6,
    selected = 7,

    /// Output type expected for querying the data of the given kind.
    pub fn OutType(comptime self: RowCellsData) type {
        return switch (self) {
            .invalid => void,
            .raw => page.Cell.C,
            .style => style_c.Style,
            .graphemes_len => u32,
            .graphemes_buf => u32,
            .bg_color, .fg_color => colorpkg.RGB.C,
            .selected => bool,
        };
    }
};

pub fn row_cells_get(
    cells_: RowCells,
    data: RowCellsData,
    out: ?*anyopaque,
) callconv(lib.calling_conv) Result {
    if (comptime std.debug.runtime_safety) {
        _ = std.meta.intToEnum(RowCellsData, @intFromEnum(data)) catch {
            log.warn("render_state_row_cells_get invalid data value={d}", .{@intFromEnum(data)});
            return .invalid_value;
        };
    }

    if (comptime build_options.lib_vt_rust) {
        const has_cell = if (cells_) |cells| cells.x != null else false;
        const result: Result = @enumFromInt(rust.ghostty_rust_render_row_cell_get(
            cells_ != null,
            has_cell,
            @intFromEnum(data),
            out != null,
        ));
        if (result != .success) return result;
    }

    return switch (data) {
        .invalid => .invalid_value,
        inline else => |comptime_data| rowCellsGetTyped(
            cells_,
            comptime_data,
            @ptrCast(@alignCast(out)),
        ),
    };
}

pub fn row_cells_get_multi(
    cells_: RowCells,
    count: usize,
    keys: ?[*]const RowCellsData,
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
        const cells = cells_ orelse return .invalid_value;
        const x = cells.x orelse return .invalid_value;
        const cell = cells.raws[x];
        const extra = if (cell.hasGrapheme()) cells.graphemes[x] else &[_]u21{};
        const s: Style = if (cell.hasStyling()) cells.styles[x] else .{};
        const fg_color = style_c.Color.fromColor(s.fg_color);
        const bg_color = style_c.Color.fromColor(s.bg_color);
        const underline_color = style_c.Color.fromColor(s.underline_color);
        const sel = cells.selection orelse .{ 0, 0 };

        return @enumFromInt(rust.ghostty_rust_render_row_cell_get_multi(
            count,
            k,
            v,
            out_written,
            cell.cval(),
            if (extra.len > 0) extra.ptr else null,
            extra.len,
            &fg_color,
            &bg_color,
            &underline_color,
            s.flags.bold,
            s.flags.italic,
            s.flags.faint,
            s.flags.blink,
            s.flags.inverse,
            s.flags.invisible,
            s.flags.strikethrough,
            s.flags.overline,
            @intFromEnum(s.flags.underline),
            cellPaletteColor(cell, cells.palette),
            stylePaletteColor(s.fg_color, cells.palette),
            stylePaletteColor(s.bg_color, cells.palette),
            cells.selection != null,
            sel[0],
            sel[1],
            x,
        ));
    }

    for (0..count) |i| {
        const result = row_cells_get(cells_, k[i], v[i]);
        if (result != .success) {
            if (out_written) |w| w.* = i;
            return result;
        }
    }
    if (out_written) |w| w.* = count;
    return .success;
}

fn rowCellsGetTyped(
    cells_: RowCells,
    comptime data: RowCellsData,
    out: *data.OutType(),
) Result {
    const cells = cells_ orelse return .invalid_value;
    const x = cells.x orelse return .invalid_value;
    const cell = cells.raws[x];
    const s: Style = if (cell.hasStyling()) cells.styles[x] else .{};

    if (comptime build_options.lib_vt_rust) {
        switch (data) {
            .style => {
                const fg_color = style_c.Color.fromColor(s.fg_color);
                const bg_color = style_c.Color.fromColor(s.bg_color);
                const underline_color = style_c.Color.fromColor(s.underline_color);
                return @enumFromInt(rust.ghostty_rust_render_row_cell_get_style(
                    &fg_color,
                    &bg_color,
                    &underline_color,
                    s.flags.bold,
                    s.flags.italic,
                    s.flags.faint,
                    s.flags.blink,
                    s.flags.inverse,
                    s.flags.invisible,
                    s.flags.strikethrough,
                    s.flags.overline,
                    @intFromEnum(s.flags.underline),
                    out,
                ));
            },
            .raw,
            .graphemes_len,
            .graphemes_buf,
            => {
                const extra = if (cell.hasGrapheme()) cells.graphemes[x] else &[_]u21{};
                return @enumFromInt(rust.ghostty_rust_render_row_cell_get_text(
                    @intFromEnum(data),
                    cell.cval(),
                    if (extra.len > 0) extra.ptr else null,
                    extra.len,
                    @ptrCast(out),
                ));
            },
            .bg_color,
            .fg_color,
            => {
                const fg_color = style_c.Color.fromColor(s.fg_color);
                const bg_color = style_c.Color.fromColor(s.bg_color);
                return @enumFromInt(rust.ghostty_rust_render_row_cell_get_color(
                    @intFromEnum(data),
                    cell.cval(),
                    &fg_color,
                    &bg_color,
                    cellPaletteColor(cell, cells.palette),
                    stylePaletteColor(s.fg_color, cells.palette),
                    stylePaletteColor(s.bg_color, cells.palette),
                    @ptrCast(out),
                ));
            },
            else => {},
        }
    }

    switch (data) {
        .invalid => return .invalid_value,
        .raw => out.* = cell.cval(),
        .style => out.* = style_c.Style.fromStyle(s),
        .graphemes_len => {
            if (!cell.hasText()) {
                out.* = 0;
                return .success;
            }
            const extra = if (cell.hasGrapheme()) cells.graphemes[x] else &[_]u21{};
            out.* = @intCast(1 + extra.len);
        },
        .graphemes_buf => {
            if (!cell.hasText()) return .success;
            const extra = if (cell.hasGrapheme()) cells.graphemes[x] else &[_]u21{};
            const buf: [*]u32 = @ptrCast(out);
            buf[0] = cell.codepoint();
            for (extra, 1..) |cp, i| {
                buf[i] = cp;
            }
        },
        .bg_color => {
            const bg = s.bg(&cell, cells.palette) orelse return .invalid_value;
            out.* = bg.cval();
        },
        .fg_color => {
            if (s.fg_color == .none) return .invalid_value;
            const fg = s.fg(.{ .default = .{}, .palette = cells.palette });
            out.* = fg.cval();
        },
        .selected => {
            if (comptime build_options.lib_vt_rust) {
                const sel = cells.selection orelse .{ 0, 0 };
                return @enumFromInt(rust.ghostty_rust_render_row_cell_selected(
                    cells.selection != null,
                    sel[0],
                    sel[1],
                    x,
                    out,
                ));
            }

            out.* = if (cells.selection) |sel|
                x >= sel[0] and x <= sel[1]
            else
                false;
        },
    }

    return .success;
}

fn cellPaletteColor(cell: page.Cell, palette: *const colorpkg.Palette) colorpkg.RGB.C {
    return switch (cell.content_tag) {
        .bg_color_palette => palette[cell.content.color_palette].cval(),
        else => .{ .r = 0, .g = 0, .b = 0 },
    };
}

fn stylePaletteColor(style_color: Style.Color, palette: *const colorpkg.Palette) colorpkg.RGB.C {
    return switch (style_color) {
        .palette => |idx| palette[idx].cval(),
        else => .{ .r = 0, .g = 0, .b = 0 },
    };
}

/// C: GhosttyRenderStateRowData
pub const RowData = enum(c_int) {
    invalid = 0,
    dirty = 1,
    raw = 2,
    cells = 3,
    selection = 4,

    /// Output type expected for querying the data of the given kind.
    pub fn OutType(comptime self: RowData) type {
        return switch (self) {
            .invalid => void,
            .dirty => bool,
            .raw => row.CRow,
            .cells => RowCells,
            .selection => RowSelection,
        };
    }
};

/// C: GhosttyRenderStateRowOption
pub const RowOption = enum(c_int) {
    dirty = 0,

    /// Input type expected for setting the option.
    pub fn InType(comptime self: RowOption) type {
        return switch (self) {
            .dirty => bool,
        };
    }
};

pub fn row_get(
    iterator_: RowIterator,
    data: RowData,
    out: ?*anyopaque,
) callconv(lib.calling_conv) Result {
    if (comptime std.debug.runtime_safety) {
        _ = std.meta.intToEnum(RowData, @intFromEnum(data)) catch {
            log.warn("render_state_row_get invalid data value={d}", .{@intFromEnum(data)});
            return .invalid_value;
        };
    }

    if (comptime build_options.lib_vt_rust) {
        const has_row = if (iterator_) |it| it.y != null else false;
        const out_size = if (data == .selection and out != null)
            @as(*const RowSelection, @ptrCast(@alignCast(out))).size
        else
            0;
        const result: Result = @enumFromInt(rust.ghostty_rust_render_row_get(
            iterator_ != null,
            has_row,
            @intFromEnum(data),
            out != null,
            out_size,
        ));
        if (result != .success) return result;
    }

    return switch (data) {
        .invalid => .invalid_value,
        inline else => |comptime_data| rowGetTyped(
            iterator_,
            comptime_data,
            @ptrCast(@alignCast(out)),
        ),
    };
}

pub fn row_get_multi(
    iterator_: RowIterator,
    count: usize,
    keys: ?[*]const RowData,
    values: ?[*]?*anyopaque,
    out_written: ?*usize,
) callconv(lib.calling_conv) Result {
    const k = keys orelse return .invalid_value;
    const v = values orelse return .invalid_value;

    if (comptime build_options.lib_vt_rust) {
        const it = iterator_ orelse return .invalid_value;
        const y = it.y orelse return .invalid_value;

        var has_cells = false;
        for (0..count) |i| {
            if (k[i] == .cells) {
                has_cells = true;
                break;
            }
        }

        if (!has_cells) {
            const sel = it.selection[y] orelse .{ 0, 0 };
            return @enumFromInt(rust.ghostty_rust_render_row_get_multi(
                count,
                k,
                v,
                out_written,
                it.raws[y].cval(),
                it.dirty[y],
                it.selection[y] != null,
                sel[0],
                sel[1],
            ));
        }
    }

    for (0..count) |i| {
        const result = row_get(iterator_, k[i], v[i]);
        if (result != .success) {
            if (out_written) |w| w.* = i;
            return result;
        }
    }
    if (out_written) |w| w.* = count;
    return .success;
}

fn rowGetTyped(
    iterator_: RowIterator,
    comptime data: RowData,
    out: *data.OutType(),
) Result {
    const it = iterator_ orelse return .invalid_value;
    const y = it.y orelse return .invalid_value;

    if (comptime build_options.lib_vt_rust) {
        switch (data) {
            .dirty,
            .raw,
            .selection,
            => {
                const sel = it.selection[y] orelse .{ 0, 0 };
                return @enumFromInt(rust.ghostty_rust_render_row_get_data(
                    @intFromEnum(data),
                    it.raws[y].cval(),
                    it.dirty[y],
                    it.selection[y] != null,
                    sel[0],
                    sel[1],
                    if (data == .selection) out.size else 0,
                    @ptrCast(out),
                ));
            },
            else => {},
        }
    }

    switch (data) {
        .invalid => return .invalid_value,
        .dirty => out.* = it.dirty[y],
        .raw => out.* = it.raws[y].cval(),
        .cells => {
            const cells = out.* orelse return .invalid_value;
            const cell_data = it.cells[y].slice();
            cells.* = .{
                .alloc = cells.alloc,
                .x = null,
                .raws = cell_data.items(.raw),
                .graphemes = cell_data.items(.grapheme),
                .styles = cell_data.items(.style),
                .selection = it.selection[y],
                .palette = it.palette,
            };
        },
        .selection => {
            const out_size = out.size;
            if (out_size < @sizeOf(RowSelection)) return .invalid_value;

            const sel = it.selection[y] orelse return .no_value;
            out.start_x = sel[0];
            out.end_x = sel[1];
        },
    }

    return .success;
}

pub fn row_set(
    iterator_: RowIterator,
    option: RowOption,
    value: ?*const anyopaque,
) callconv(lib.calling_conv) Result {
    if (comptime std.debug.runtime_safety) {
        _ = std.meta.intToEnum(RowOption, @intFromEnum(option)) catch {
            log.warn("render_state_row_set invalid option value={d}", .{@intFromEnum(option)});
            return .invalid_value;
        };
    }

    if (comptime build_options.lib_vt_rust) {
        const has_row = if (iterator_) |it| it.y != null else false;
        const result: Result = @enumFromInt(rust.ghostty_rust_render_row_set(
            iterator_ != null,
            has_row,
            @intFromEnum(option),
            value != null,
        ));
        if (result != .success) return result;
    }

    return switch (option) {
        inline else => |comptime_option| rowSetTyped(
            iterator_,
            comptime_option,
            @ptrCast(@alignCast(value orelse return .invalid_value)),
        ),
    };
}

fn rowSetTyped(
    iterator_: RowIterator,
    comptime option: RowOption,
    value: *const option.InType(),
) Result {
    const it = iterator_ orelse return .invalid_value;
    const y = it.y orelse return .invalid_value;
    switch (option) {
        .dirty => {
            if (comptime build_options.lib_vt_rust) {
                return @enumFromInt(rust.ghostty_rust_render_row_set_dirty(
                    value.*,
                    &it.dirty[y],
                ));
            }

            it.dirty[y] = value.*;
        },
    }

    return .success;
}

test "render: new/free" {
    var state: RenderState = null;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &state,
    ));
    try testing.expect(state != null);
    free(state);
}

test "render: free null" {
    free(null);
}

test "render: update invalid value" {
    var state: RenderState = null;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &state,
    ));
    defer free(state);

    try testing.expectEqual(Result.invalid_value, update(null, null));
    try testing.expectEqual(Result.invalid_value, update(state, null));
}

test "render: get invalid value" {
    var cols: size.CellCountInt = 0;
    try testing.expectEqual(Result.invalid_value, get(null, .cols, @ptrCast(&cols)));
}

test "render: get invalid data" {
    var state: RenderState = null;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &state,
    ));
    defer free(state);

    try testing.expectEqual(Result.invalid_value, get(state, .invalid, null));
}

test "render: colors get invalid value" {
    var state: RenderState = null;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &state,
    ));
    defer free(state);

    var colors: Colors = std.mem.zeroes(Colors);
    colors.size = @sizeOf(Colors);

    try testing.expectEqual(Result.invalid_value, colors_get(null, &colors));
    try testing.expectEqual(Result.invalid_value, colors_get(state, null));

    colors.size = @sizeOf(usize) - 1;
    try testing.expectEqual(Result.invalid_value, colors_get(state, &colors));
}

test "render: get/set dirty invalid value" {
    var dirty: Dirty = .false;
    try testing.expectEqual(Result.invalid_value, get(null, .dirty, @ptrCast(&dirty)));
    const dirty_full: Dirty = .full;
    try testing.expectEqual(Result.invalid_value, set(null, .dirty, @ptrCast(&dirty_full)));
}

test "render: get/set dirty" {
    var state: RenderState = null;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &state,
    ));
    defer free(state);

    var dirty: Dirty = undefined;
    try testing.expectEqual(Result.success, get(state, .dirty, @ptrCast(&dirty)));
    try testing.expectEqual(Dirty.false, dirty);

    const dirty_partial: Dirty = .partial;
    try testing.expectEqual(Result.success, set(state, .dirty, @ptrCast(&dirty_partial)));
    try testing.expectEqual(Result.success, get(state, .dirty, @ptrCast(&dirty)));
    try testing.expectEqual(Dirty.partial, dirty);

    const dirty_full: Dirty = .full;
    try testing.expectEqual(Result.success, set(state, .dirty, @ptrCast(&dirty_full)));
    try testing.expectEqual(Result.success, get(state, .dirty, @ptrCast(&dirty)));
    try testing.expectEqual(Dirty.full, dirty);
}

test "render: get colors" {
    var state: RenderState = null;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &state,
    ));
    defer free(state);

    var background: colorpkg.RGB.C = undefined;
    try testing.expectEqual(Result.success, get(state, .color_background, @ptrCast(&background)));
    try testing.expectEqualDeep(colorpkg.RGB.C{ .r = 0, .g = 0, .b = 0 }, background);

    var foreground: colorpkg.RGB.C = undefined;
    try testing.expectEqual(Result.success, get(state, .color_foreground, @ptrCast(&foreground)));
    try testing.expectEqualDeep(colorpkg.RGB.C{ .r = 0xff, .g = 0xff, .b = 0xff }, foreground);

    var cursor_has_value: bool = undefined;
    try testing.expectEqual(Result.success, get(state, .color_cursor_has_value, @ptrCast(&cursor_has_value)));
    try testing.expect(!cursor_has_value);

    var cursor: colorpkg.RGB.C = undefined;
    try testing.expectEqual(Result.invalid_value, get(state, .color_cursor, @ptrCast(&cursor)));

    var palette: colorpkg.PaletteC = undefined;
    try testing.expectEqual(Result.success, get(state, .color_palette, @ptrCast(&palette)));
    try testing.expectEqualDeep(colorpkg.paletteCval(&colorpkg.default), palette);
}

test "render: set null value" {
    var state: RenderState = null;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &state,
    ));
    defer free(state);

    try testing.expectEqual(Result.invalid_value, set(state, .dirty, null));
}

test "render: row iterator get invalid value" {
    var iterator: RowIterator = null;
    try testing.expectEqual(Result.success, row_iterator_new(
        &lib.alloc.test_allocator,
        &iterator,
    ));
    defer row_iterator_free(iterator);

    try testing.expectEqual(Result.invalid_value, get(null, .row_iterator, @ptrCast(&iterator)));
}

test "render: row iterator new/free" {
    var terminal: terminal_c.Terminal = null;
    try testing.expectEqual(Result.success, terminal_c.new(
        &lib.alloc.test_allocator,
        &terminal,
        .{
            .cols = 80,
            .rows = 24,
            .max_scrollback = 10_000,
        },
    ));
    defer terminal_c.free(terminal);

    var state: RenderState = null;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &state,
    ));
    defer free(state);

    try testing.expectEqual(Result.success, update(state, terminal));

    var iterator: RowIterator = null;
    try testing.expectEqual(Result.success, row_iterator_new(
        &lib.alloc.test_allocator,
        &iterator,
    ));
    defer row_iterator_free(iterator);

    try testing.expect(iterator != null);

    try testing.expectEqual(Result.success, get(state, .row_iterator, @ptrCast(&iterator)));

    const iterator_ptr = iterator.?;
    const row_data = state.?.state.row_data.slice();

    try testing.expectEqual(@as(?size.CellCountInt, null), iterator_ptr.y);
    try testing.expectEqual(row_data.items(.raw).len, iterator_ptr.raws.len);
    try testing.expectEqual(row_data.items(.cells).len, iterator_ptr.cells.len);
    try testing.expectEqual(row_data.items(.selection).len, iterator_ptr.selection.len);
    try testing.expectEqual(row_data.items(.dirty).len, iterator_ptr.dirty.len);
}

test "render: row iterator free null" {
    row_iterator_free(null);
}

test "render: row iterator next null" {
    try testing.expect(!row_iterator_next(null));
}

test "render: row get null" {
    var dirty: bool = undefined;
    try testing.expectEqual(Result.invalid_value, row_get(null, .dirty, @ptrCast(&dirty)));
}

test "render: row get invalid data" {
    var terminal: terminal_c.Terminal = null;
    try testing.expectEqual(Result.success, terminal_c.new(
        &lib.alloc.test_allocator,
        &terminal,
        .{
            .cols = 80,
            .rows = 24,
            .max_scrollback = 10_000,
        },
    ));
    defer terminal_c.free(terminal);

    var state: RenderState = null;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &state,
    ));
    defer free(state);

    try testing.expectEqual(Result.success, update(state, terminal));

    var iterator: RowIterator = null;
    try testing.expectEqual(Result.success, row_iterator_new(
        &lib.alloc.test_allocator,
        &iterator,
    ));
    defer row_iterator_free(iterator);

    try testing.expectEqual(Result.success, get(state, .row_iterator, @ptrCast(&iterator)));
    try testing.expect(row_iterator_next(iterator));
    try testing.expectEqual(Result.invalid_value, row_get(iterator, .invalid, null));
}

test "render: row set null" {
    const dirty = false;
    try testing.expectEqual(Result.invalid_value, row_set(null, .dirty, @ptrCast(&dirty)));
}

test "render: row set before iteration" {
    var terminal: terminal_c.Terminal = null;
    try testing.expectEqual(Result.success, terminal_c.new(
        &lib.alloc.test_allocator,
        &terminal,
        .{
            .cols = 80,
            .rows = 24,
            .max_scrollback = 10_000,
        },
    ));
    defer terminal_c.free(terminal);

    var state: RenderState = null;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &state,
    ));
    defer free(state);

    try testing.expectEqual(Result.success, update(state, terminal));

    var iterator: RowIterator = null;
    try testing.expectEqual(Result.success, row_iterator_new(
        &lib.alloc.test_allocator,
        &iterator,
    ));
    defer row_iterator_free(iterator);

    try testing.expectEqual(Result.success, get(state, .row_iterator, @ptrCast(&iterator)));
    const dirty = false;
    try testing.expectEqual(Result.invalid_value, row_set(iterator, .dirty, @ptrCast(&dirty)));
}

test "render: row get before iteration" {
    var terminal: terminal_c.Terminal = null;
    try testing.expectEqual(Result.success, terminal_c.new(
        &lib.alloc.test_allocator,
        &terminal,
        .{
            .cols = 80,
            .rows = 24,
            .max_scrollback = 10_000,
        },
    ));
    defer terminal_c.free(terminal);

    var state: RenderState = null;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &state,
    ));
    defer free(state);

    try testing.expectEqual(Result.success, update(state, terminal));

    var iterator: RowIterator = null;
    try testing.expectEqual(Result.success, row_iterator_new(
        &lib.alloc.test_allocator,
        &iterator,
    ));
    defer row_iterator_free(iterator);

    try testing.expectEqual(Result.success, get(state, .row_iterator, @ptrCast(&iterator)));
    var dirty: bool = undefined;
    try testing.expectEqual(Result.invalid_value, row_get(iterator, .dirty, @ptrCast(&dirty)));
}

test "render: row get raw and scalar multi" {
    var raw_rows = [_]page.Row{@bitCast(@as(u64, 0))};
    raw_rows[0].wrap = true;
    var dirty_rows = [_]bool{true};
    var selection_rows = [_]?[2]size.CellCountInt{.{ 1, 3 }};
    var palette: colorpkg.Palette = undefined;

    var wrapper: RowIteratorWrapper = .{
        .alloc = lib.alloc.default(&lib.alloc.test_allocator),
        .y = 0,
        .raws = &raw_rows,
        .cells = undefined,
        .selection = &selection_rows,
        .dirty = &dirty_rows,
        .palette = &palette,
    };
    const it: RowIterator = &wrapper;

    var raw_value: row.CRow = 0;
    try testing.expectEqual(Result.success, row_get(it, .raw, @ptrCast(&raw_value)));
    try testing.expectEqual(raw_rows[0].cval(), raw_value);

    var dirty = false;
    var selection: RowSelection = .{};
    const keys = [_]RowData{ .dirty, .raw, .selection };
    var values = [_]?*anyopaque{
        @ptrCast(&dirty),
        @ptrCast(&raw_value),
        @ptrCast(&selection),
    };
    var written: usize = 0;

    raw_value = 0;
    try testing.expectEqual(Result.success, row_get_multi(it, keys.len, &keys, &values, &written));
    try testing.expectEqual(keys.len, written);
    try testing.expect(dirty);
    try testing.expectEqual(raw_rows[0].cval(), raw_value);
    try testing.expectEqual(@as(size.CellCountInt, 1), selection.start_x);
    try testing.expectEqual(@as(size.CellCountInt, 3), selection.end_x);
}

test "render: row get/set dirty" {
    var terminal: terminal_c.Terminal = null;
    try testing.expectEqual(Result.success, terminal_c.new(
        &lib.alloc.test_allocator,
        &terminal,
        .{
            .cols = 80,
            .rows = 24,
            .max_scrollback = 10_000,
        },
    ));
    defer terminal_c.free(terminal);

    var state: RenderState = null;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &state,
    ));
    defer free(state);

    try testing.expectEqual(Result.success, update(state, terminal));

    // Dirty the first row so the iterator has at least one dirty row to observe.
    terminal_c.vt_write(terminal, "hello", 5);
    try testing.expectEqual(Result.success, update(state, terminal));

    // Create an iterator and verify it is dirty.
    var it: RowIterator = null;
    try testing.expectEqual(Result.success, row_iterator_new(
        &lib.alloc.test_allocator,
        &it,
    ));
    defer row_iterator_free(it);

    try testing.expectEqual(Result.success, get(state, .row_iterator, @ptrCast(&it)));
    try testing.expect(row_iterator_next(it));
    var dirty: bool = undefined;
    try testing.expectEqual(Result.success, row_get(it, .dirty, @ptrCast(&dirty)));
    try testing.expect(dirty);

    // Clear dirty on this row.
    const dirty_false = false;
    try testing.expectEqual(Result.success, row_set(it, .dirty, @ptrCast(&dirty_false)));

    // It should not be dirty anymore.
    var it2: RowIterator = null;
    try testing.expectEqual(Result.success, row_iterator_new(
        &lib.alloc.test_allocator,
        &it2,
    ));
    defer row_iterator_free(it2);

    try testing.expectEqual(Result.success, get(state, .row_iterator, @ptrCast(&it2)));
    try testing.expect(row_iterator_next(it2));
    try testing.expectEqual(Result.success, row_get(it2, .dirty, @ptrCast(&dirty)));
    try testing.expect(!dirty);
}

test "render: row get selection" {
    var terminal: terminal_c.Terminal = null;
    try testing.expectEqual(Result.success, terminal_c.new(
        &lib.alloc.test_allocator,
        &terminal,
        .{
            .cols = 10,
            .rows = 3,
            .max_scrollback = 10_000,
        },
    ));
    defer terminal_c.free(terminal);

    const t = terminal.?.terminal;
    const screen = t.screens.active;
    try screen.select(.init(
        screen.pages.pin(.{ .active = .{ .x = 2, .y = 1 } }).?,
        screen.pages.pin(.{ .active = .{ .x = 4, .y = 1 } }).?,
        false,
    ));

    var state: RenderState = null;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &state,
    ));
    defer free(state);

    try testing.expectEqual(Result.success, update(state, terminal));

    var it: RowIterator = null;
    try testing.expectEqual(Result.success, row_iterator_new(
        &lib.alloc.test_allocator,
        &it,
    ));
    defer row_iterator_free(it);

    try testing.expectEqual(Result.success, get(state, .row_iterator, @ptrCast(&it)));

    var sel: RowSelection = .{};
    try testing.expect(row_iterator_next(it));
    try testing.expectEqual(Result.no_value, row_get(it, .selection, @ptrCast(&sel)));

    try testing.expect(row_iterator_next(it));
    sel = .{};
    try testing.expectEqual(Result.success, row_get(it, .selection, @ptrCast(&sel)));
    try testing.expectEqual(@as(u16, 2), sel.start_x);
    try testing.expectEqual(@as(u16, 4), sel.end_x);

    sel.size = @sizeOf(usize) - 1;
    try testing.expectEqual(Result.invalid_value, row_get(it, .selection, @ptrCast(&sel)));

    try testing.expect(row_iterator_next(it));
    sel = .{};
    try testing.expectEqual(Result.no_value, row_get(it, .selection, @ptrCast(&sel)));
}

test "render: row cells get invalid value" {
    var selected = false;
    try testing.expectEqual(Result.invalid_value, row_cells_get(null, .selected, @ptrCast(&selected)));

    const raw_cells = [_]page.Cell{};
    const graphemes = [_][]const u21{};
    const styles = [_]Style{};
    var palette: colorpkg.Palette = undefined;
    var wrapper: RowCellsWrapper = .{
        .alloc = lib.alloc.default(&lib.alloc.test_allocator),
        .x = null,
        .raws = &raw_cells,
        .graphemes = &graphemes,
        .styles = &styles,
        .selection = null,
        .palette = &palette,
    };
    const cells: RowCells = &wrapper;

    try testing.expectEqual(Result.invalid_value, row_cells_get(cells, .invalid, @ptrCast(&selected)));
    try testing.expectEqual(Result.invalid_value, row_cells_get(cells, .selected, @ptrCast(&selected)));
}

test "render: row cells get selected" {
    var terminal: terminal_c.Terminal = null;
    try testing.expectEqual(Result.success, terminal_c.new(
        &lib.alloc.test_allocator,
        &terminal,
        .{
            .cols = 10,
            .rows = 3,
            .max_scrollback = 10_000,
        },
    ));
    defer terminal_c.free(terminal);

    const t = terminal.?.terminal;
    const screen = t.screens.active;
    try screen.select(.init(
        screen.pages.pin(.{ .active = .{ .x = 2, .y = 1 } }).?,
        screen.pages.pin(.{ .active = .{ .x = 4, .y = 1 } }).?,
        false,
    ));

    var state: RenderState = null;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &state,
    ));
    defer free(state);

    try testing.expectEqual(Result.success, update(state, terminal));

    var it: RowIterator = null;
    try testing.expectEqual(Result.success, row_iterator_new(
        &lib.alloc.test_allocator,
        &it,
    ));
    defer row_iterator_free(it);

    var cells: RowCells = null;
    try testing.expectEqual(Result.success, row_cells_new(
        &lib.alloc.test_allocator,
        &cells,
    ));
    defer row_cells_free(cells);

    try testing.expectEqual(Result.success, get(state, .row_iterator, @ptrCast(&it)));

    try testing.expect(row_iterator_next(it));
    try testing.expectEqual(Result.success, row_get(it, .cells, @ptrCast(&cells)));

    var selected: bool = true;
    try testing.expectEqual(Result.success, row_cells_select(cells, 0));
    try testing.expectEqual(Result.success, row_cells_get(cells, .selected, @ptrCast(&selected)));
    try testing.expect(!selected);

    try testing.expect(row_iterator_next(it));
    try testing.expectEqual(Result.success, row_get(it, .cells, @ptrCast(&cells)));

    try testing.expectEqual(Result.success, row_cells_select(cells, 1));
    try testing.expectEqual(Result.success, row_cells_get(cells, .selected, @ptrCast(&selected)));
    try testing.expect(!selected);

    try testing.expectEqual(Result.success, row_cells_select(cells, 2));
    try testing.expectEqual(Result.success, row_cells_get(cells, .selected, @ptrCast(&selected)));
    try testing.expect(selected);

    try testing.expectEqual(Result.success, row_cells_select(cells, 4));
    try testing.expectEqual(Result.success, row_cells_get(cells, .selected, @ptrCast(&selected)));
    try testing.expect(selected);

    try testing.expectEqual(Result.success, row_cells_select(cells, 5));
    try testing.expectEqual(Result.success, row_cells_get(cells, .selected, @ptrCast(&selected)));
    try testing.expect(!selected);

    try testing.expectEqual(Result.success, row_cells_select(cells, 3));
    selected = false;
    var written: usize = 0;
    const keys = [_]RowCellsData{.selected};
    var values = [_]?*anyopaque{@ptrCast(&selected)};
    try testing.expectEqual(Result.success, row_cells_get_multi(cells, keys.len, &keys, &values, &written));
    try testing.expectEqual(keys.len, written);
    try testing.expect(selected);
}

test "render: row cells text metadata" {
    var raw = [_]page.Cell{
        page.Cell.init('e'),
        page.Cell.init('x'),
        @bitCast(@as(u64, 0)),
    };
    raw[0].content_tag = .codepoint_grapheme;

    const extra0 = [_]u21{0x0301};
    const extra_empty = [_]u21{};
    const graphemes = [_][]const u21{
        &extra0,
        &extra_empty,
        &extra_empty,
    };
    const styles = [_]Style{ .{}, .{}, .{} };
    var palette: colorpkg.Palette = undefined;

    var wrapper: RowCellsWrapper = .{
        .alloc = lib.alloc.default(&lib.alloc.test_allocator),
        .x = null,
        .raws = &raw,
        .graphemes = &graphemes,
        .styles = &styles,
        .selection = null,
        .palette = &palette,
    };
    const cells: RowCells = &wrapper;

    try testing.expectEqual(Result.success, row_cells_select(cells, 0));

    var raw_value: page.Cell.C = 0;
    try testing.expectEqual(Result.success, row_cells_get(cells, .raw, @ptrCast(&raw_value)));
    try testing.expectEqual(raw[0].cval(), raw_value);

    var len: u32 = 0;
    try testing.expectEqual(Result.success, row_cells_get(cells, .graphemes_len, @ptrCast(&len)));
    try testing.expectEqual(@as(u32, 2), len);

    var buf = [_]u32{ 0, 0 };
    try testing.expectEqual(Result.success, row_cells_get(cells, .graphemes_buf, @ptrCast(&buf)));
    try testing.expectEqualSlices(u32, &.{ 'e', 0x0301 }, &buf);

    try testing.expectEqual(Result.success, row_cells_select(cells, 2));
    len = 99;
    try testing.expectEqual(Result.success, row_cells_get(cells, .graphemes_len, @ptrCast(&len)));
    try testing.expectEqual(@as(u32, 0), len);
}

test "render: row cells palette colors" {
    var raw = [_]page.Cell{
        page.Cell.init('A'),
        @bitCast(@as(u64, 0)),
    };
    raw[0].style_id = 1;
    raw[1].content_tag = .bg_color_palette;
    raw[1].content = .{ .color_palette = 3 };

    const extra_empty = [_]u21{};
    const graphemes = [_][]const u21{
        &extra_empty,
        &extra_empty,
    };
    const styles = [_]Style{
        .{
            .fg_color = .{ .palette = 1 },
            .bg_color = .{ .palette = 2 },
        },
        .{},
    };
    var palette = colorpkg.default;
    palette[1] = .{ .r = 10, .g = 20, .b = 30 };
    palette[2] = .{ .r = 40, .g = 50, .b = 60 };
    palette[3] = .{ .r = 70, .g = 80, .b = 90 };

    var wrapper: RowCellsWrapper = .{
        .alloc = lib.alloc.default(&lib.alloc.test_allocator),
        .x = null,
        .raws = &raw,
        .graphemes = &graphemes,
        .styles = &styles,
        .selection = null,
        .palette = &palette,
    };
    const cells: RowCells = &wrapper;

    try testing.expectEqual(Result.success, row_cells_select(cells, 0));

    var fg: colorpkg.RGB.C = undefined;
    try testing.expectEqual(Result.success, row_cells_get(cells, .fg_color, @ptrCast(&fg)));
    try testing.expectEqual(@as(u8, 10), fg.r);
    try testing.expectEqual(@as(u8, 20), fg.g);
    try testing.expectEqual(@as(u8, 30), fg.b);

    var bg: colorpkg.RGB.C = undefined;
    try testing.expectEqual(Result.success, row_cells_get(cells, .bg_color, @ptrCast(&bg)));
    try testing.expectEqual(@as(u8, 40), bg.r);
    try testing.expectEqual(@as(u8, 50), bg.g);
    try testing.expectEqual(@as(u8, 60), bg.b);

    try testing.expectEqual(Result.success, row_cells_select(cells, 1));
    try testing.expectEqual(Result.success, row_cells_get(cells, .bg_color, @ptrCast(&bg)));
    try testing.expectEqual(@as(u8, 70), bg.r);
    try testing.expectEqual(@as(u8, 80), bg.g);
    try testing.expectEqual(@as(u8, 90), bg.b);
}

test "render: row cells style" {
    var raw = [_]page.Cell{
        page.Cell.init('A'),
        page.Cell.init('B'),
    };
    raw[0].style_id = 1;

    const extra_empty = [_]u21{};
    const graphemes = [_][]const u21{
        &extra_empty,
        &extra_empty,
    };
    const styles = [_]Style{
        .{
            .fg_color = .{ .palette = 42 },
            .bg_color = .{ .rgb = .{ .r = 255, .g = 128, .b = 64 } },
            .underline_color = .none,
            .flags = .{
                .bold = true,
                .italic = true,
                .underline = .curly,
            },
        },
        .{},
    };
    var palette: colorpkg.Palette = undefined;

    var wrapper: RowCellsWrapper = .{
        .alloc = lib.alloc.default(&lib.alloc.test_allocator),
        .x = null,
        .raws = &raw,
        .graphemes = &graphemes,
        .styles = &styles,
        .selection = null,
        .palette = &palette,
    };
    const cells: RowCells = &wrapper;

    try testing.expectEqual(Result.success, row_cells_select(cells, 0));

    var styled: style_c.Style = undefined;
    try testing.expectEqual(Result.success, row_cells_get(cells, .style, @ptrCast(&styled)));
    try testing.expectEqual(@sizeOf(style_c.Style), styled.size);
    try testing.expectEqual(style_c.ColorTag.palette, styled.fg_color.tag);
    try testing.expectEqual(@as(u8, 42), styled.fg_color.value.palette);
    try testing.expectEqual(style_c.ColorTag.rgb, styled.bg_color.tag);
    try testing.expectEqual(@as(u8, 255), styled.bg_color.value.rgb.r);
    try testing.expectEqual(@as(u8, 128), styled.bg_color.value.rgb.g);
    try testing.expectEqual(@as(u8, 64), styled.bg_color.value.rgb.b);
    try testing.expectEqual(style_c.ColorTag.none, styled.underline_color.tag);
    try testing.expect(styled.bold);
    try testing.expect(styled.italic);
    try testing.expect(!styled.faint);
    try testing.expectEqual(@as(c_int, 3), styled.underline);
    try testing.expect(!style_c.style_is_default(&styled));

    try testing.expectEqual(Result.success, row_cells_select(cells, 1));
    var default_style: style_c.Style = undefined;
    try testing.expectEqual(Result.success, row_cells_get(cells, .style, @ptrCast(&default_style)));
    try testing.expect(style_c.style_is_default(&default_style));
}

test "render: row cells get_multi mixed data" {
    var raw = [_]page.Cell{page.Cell.init('A')};
    raw[0].style_id = 1;
    raw[0].content_tag = .codepoint_grapheme;

    const extra0 = [_]u21{0x0301};
    const graphemes = [_][]const u21{&extra0};
    const styles = [_]Style{.{
        .fg_color = .{ .rgb = .{ .r = 10, .g = 20, .b = 30 } },
        .bg_color = .{ .rgb = .{ .r = 40, .g = 50, .b = 60 } },
        .flags = .{ .bold = true },
    }};
    var palette: colorpkg.Palette = undefined;

    var wrapper: RowCellsWrapper = .{
        .alloc = lib.alloc.default(&lib.alloc.test_allocator),
        .x = null,
        .raws = &raw,
        .graphemes = &graphemes,
        .styles = &styles,
        .selection = .{ 0, 0 },
        .palette = &palette,
    };
    const cells: RowCells = &wrapper;

    try testing.expectEqual(Result.success, row_cells_select(cells, 0));

    var raw_value: page.Cell.C = 0;
    var styled: style_c.Style = undefined;
    var len: u32 = 0;
    var fg: colorpkg.RGB.C = undefined;
    var bg: colorpkg.RGB.C = undefined;
    var selected = false;
    const keys = [_]RowCellsData{
        .raw,
        .style,
        .graphemes_len,
        .fg_color,
        .bg_color,
        .selected,
    };
    var values = [_]?*anyopaque{
        @ptrCast(&raw_value),
        @ptrCast(&styled),
        @ptrCast(&len),
        @ptrCast(&fg),
        @ptrCast(&bg),
        @ptrCast(&selected),
    };
    var written: usize = 0;

    try testing.expectEqual(Result.success, row_cells_get_multi(cells, keys.len, &keys, &values, &written));
    try testing.expectEqual(keys.len, written);
    try testing.expectEqual(raw[0].cval(), raw_value);
    try testing.expectEqual(@as(u32, 2), len);
    try testing.expectEqual(style_c.ColorTag.rgb, styled.fg_color.tag);
    try testing.expect(styled.bold);
    try testing.expectEqual(@as(u8, 10), fg.r);
    try testing.expectEqual(@as(u8, 20), fg.g);
    try testing.expectEqual(@as(u8, 30), fg.b);
    try testing.expectEqual(@as(u8, 40), bg.r);
    try testing.expectEqual(@as(u8, 50), bg.g);
    try testing.expectEqual(@as(u8, 60), bg.b);
    try testing.expect(selected);
}

test "render: row iterator next" {
    var terminal: terminal_c.Terminal = null;
    try testing.expectEqual(Result.success, terminal_c.new(
        &lib.alloc.test_allocator,
        &terminal,
        .{
            .cols = 80,
            .rows = 24,
            .max_scrollback = 10_000,
        },
    ));
    defer terminal_c.free(terminal);

    var state: RenderState = null;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &state,
    ));
    defer free(state);

    try testing.expectEqual(Result.success, update(state, terminal));

    var iterator: RowIterator = null;
    try testing.expectEqual(Result.success, row_iterator_new(
        &lib.alloc.test_allocator,
        &iterator,
    ));
    defer row_iterator_free(iterator);

    try testing.expectEqual(Result.success, get(state, .row_iterator, @ptrCast(&iterator)));

    const rows = state.?.state.rows;
    if (rows == 0) {
        try testing.expect(!row_iterator_next(iterator));
        return;
    }

    try testing.expect(row_iterator_next(iterator));
    try testing.expectEqual(@as(?size.CellCountInt, 0), iterator.?.y);

    var i: size.CellCountInt = 1;
    while (i < rows) : (i += 1) {
        try testing.expect(row_iterator_next(iterator));
        try testing.expectEqual(@as(?size.CellCountInt, i), iterator.?.y);
    }

    try testing.expect(!row_iterator_next(iterator));
    try testing.expectEqual(@as(?size.CellCountInt, rows - 1), iterator.?.y);
}

test "render: update" {
    var terminal: terminal_c.Terminal = null;
    try testing.expectEqual(Result.success, terminal_c.new(
        &lib.alloc.test_allocator,
        &terminal,
        .{
            .cols = 80,
            .rows = 24,
            .max_scrollback = 10_000,
        },
    ));
    defer terminal_c.free(terminal);

    var state: RenderState = null;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &state,
    ));
    defer free(state);

    try testing.expectEqual(Result.success, update(state, terminal));

    terminal_c.vt_write(terminal, "hello", 5);
    try testing.expectEqual(Result.success, update(state, terminal));

    var cols: size.CellCountInt = 0;
    var rows_val: size.CellCountInt = 0;
    try testing.expectEqual(Result.success, get(state, .cols, @ptrCast(&cols)));
    try testing.expectEqual(Result.success, get(state, .rows, @ptrCast(&rows_val)));
    try testing.expectEqual(@as(size.CellCountInt, 80), cols);
    try testing.expectEqual(@as(size.CellCountInt, 24), rows_val);
}

test "render: cursor primitive getters" {
    var terminal: terminal_c.Terminal = null;
    try testing.expectEqual(Result.success, terminal_c.new(
        &lib.alloc.test_allocator,
        &terminal,
        .{
            .cols = 10,
            .rows = 2,
            .max_scrollback = 10_000,
        },
    ));
    defer terminal_c.free(terminal);

    terminal_c.vt_write(terminal, "A\r\nB\r\nC\r\nD\r\n", 12);

    var state: RenderState = null;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &state,
    ));
    defer free(state);

    try testing.expectEqual(Result.success, update(state, terminal));

    var has_viewport: bool = false;
    try testing.expectEqual(Result.success, get(state, .cursor_viewport_has_value, @ptrCast(&has_viewport)));
    try testing.expect(has_viewport);

    var x: size.CellCountInt = 99;
    var y: size.CellCountInt = 99;
    var wide_tail = true;
    try testing.expectEqual(Result.success, get(state, .cursor_viewport_x, @ptrCast(&x)));
    try testing.expectEqual(Result.success, get(state, .cursor_viewport_y, @ptrCast(&y)));
    try testing.expectEqual(Result.success, get(state, .cursor_viewport_wide_tail, @ptrCast(&wide_tail)));
    try testing.expectEqual(@as(size.CellCountInt, 0), x);
    try testing.expectEqual(@as(size.CellCountInt, 1), y);
    try testing.expect(!wide_tail);

    terminal.?.terminal.scrollViewport(.top);
    try testing.expectEqual(Result.success, update(state, terminal));

    has_viewport = true;
    try testing.expectEqual(Result.success, get(state, .cursor_viewport_has_value, @ptrCast(&has_viewport)));
    try testing.expect(!has_viewport);
    try testing.expectEqual(Result.invalid_value, get(state, .cursor_viewport_x, @ptrCast(&x)));
    try testing.expectEqual(Result.invalid_value, get(state, .cursor_viewport_y, @ptrCast(&y)));
    try testing.expectEqual(Result.invalid_value, get(state, .cursor_viewport_wide_tail, @ptrCast(&wide_tail)));
}

test "render: colors get" {
    var terminal: terminal_c.Terminal = null;
    try testing.expectEqual(Result.success, terminal_c.new(
        &lib.alloc.test_allocator,
        &terminal,
        .{
            .cols = 80,
            .rows = 24,
            .max_scrollback = 10_000,
        },
    ));
    defer terminal_c.free(terminal);

    var state: RenderState = null;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &state,
    ));
    defer free(state);

    try testing.expectEqual(Result.success, update(state, terminal));

    var colors: Colors = std.mem.zeroes(Colors);
    colors.size = @sizeOf(Colors);
    try testing.expectEqual(Result.success, colors_get(state, &colors));

    const state_colors = &state.?.state.colors;
    try testing.expectEqual(state_colors.background.cval(), colors.background);
    try testing.expectEqual(state_colors.foreground.cval(), colors.foreground);

    if (state_colors.cursor) |cursor| {
        try testing.expect(colors.cursor_has_value);
        try testing.expectEqual(cursor.cval(), colors.cursor);
    } else {
        try testing.expect(!colors.cursor_has_value);
    }

    for (state_colors.palette, colors.palette) |expected, actual| {
        try testing.expectEqual(expected.cval(), actual);
    }
}

test "render: row cells bg_color no background" {
    var terminal: terminal_c.Terminal = null;
    try testing.expectEqual(Result.success, terminal_c.new(
        &lib.alloc.test_allocator,
        &terminal,
        .{
            .cols = 80,
            .rows = 24,
            .max_scrollback = 10_000,
        },
    ));
    defer terminal_c.free(terminal);

    // Write plain text (no background color set).
    terminal_c.vt_write(terminal, "hello", 5);

    var state: RenderState = null;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &state,
    ));
    defer free(state);

    try testing.expectEqual(Result.success, update(state, terminal));

    var it: RowIterator = null;
    try testing.expectEqual(Result.success, row_iterator_new(
        &lib.alloc.test_allocator,
        &it,
    ));
    defer row_iterator_free(it);

    try testing.expectEqual(Result.success, get(state, .row_iterator, @ptrCast(&it)));
    try testing.expect(row_iterator_next(it));

    var cells: RowCells = null;
    try testing.expectEqual(Result.success, row_cells_new(
        &lib.alloc.test_allocator,
        &cells,
    ));
    defer row_cells_free(cells);

    try testing.expectEqual(Result.success, row_get(it, .cells, @ptrCast(&cells)));
    try testing.expect(row_cells_next(cells));

    // No background set, should return invalid_value.
    var bg: colorpkg.RGB.C = undefined;
    try testing.expectEqual(Result.invalid_value, row_cells_get(cells, .bg_color, @ptrCast(&bg)));
}

test "render: row cells bg_color from style" {
    var terminal: terminal_c.Terminal = null;
    try testing.expectEqual(Result.success, terminal_c.new(
        &lib.alloc.test_allocator,
        &terminal,
        .{
            .cols = 80,
            .rows = 24,
            .max_scrollback = 10_000,
        },
    ));
    defer terminal_c.free(terminal);

    // Set an RGB background via SGR 48;2;R;G;B and write text.
    terminal_c.vt_write(terminal, "\x1b[48;2;10;20;30mA", 18);

    var state: RenderState = null;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &state,
    ));
    defer free(state);

    try testing.expectEqual(Result.success, update(state, terminal));

    var it: RowIterator = null;
    try testing.expectEqual(Result.success, row_iterator_new(
        &lib.alloc.test_allocator,
        &it,
    ));
    defer row_iterator_free(it);

    try testing.expectEqual(Result.success, get(state, .row_iterator, @ptrCast(&it)));
    try testing.expect(row_iterator_next(it));

    var cells: RowCells = null;
    try testing.expectEqual(Result.success, row_cells_new(
        &lib.alloc.test_allocator,
        &cells,
    ));
    defer row_cells_free(cells);

    try testing.expectEqual(Result.success, row_get(it, .cells, @ptrCast(&cells)));
    try testing.expect(row_cells_next(cells));

    var bg: colorpkg.RGB.C = undefined;
    try testing.expectEqual(Result.success, row_cells_get(cells, .bg_color, @ptrCast(&bg)));
    try testing.expectEqual(@as(u8, 10), bg.r);
    try testing.expectEqual(@as(u8, 20), bg.g);
    try testing.expectEqual(@as(u8, 30), bg.b);
}

test "render: row cells bg_color from content tag" {
    var terminal: terminal_c.Terminal = null;
    try testing.expectEqual(Result.success, terminal_c.new(
        &lib.alloc.test_allocator,
        &terminal,
        .{
            .cols = 80,
            .rows = 24,
            .max_scrollback = 10_000,
        },
    ));
    defer terminal_c.free(terminal);

    // Set an RGB background and then erase the line. The erased cells
    // should carry the background color via the content tag (bg_color_rgb)
    // rather than through the style.
    terminal_c.vt_write(terminal, "\x1b[48;2;10;20;30m\x1b[2K", 21);

    var state: RenderState = null;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &state,
    ));
    defer free(state);

    try testing.expectEqual(Result.success, update(state, terminal));

    var it: RowIterator = null;
    try testing.expectEqual(Result.success, row_iterator_new(
        &lib.alloc.test_allocator,
        &it,
    ));
    defer row_iterator_free(it);

    try testing.expectEqual(Result.success, get(state, .row_iterator, @ptrCast(&it)));
    try testing.expect(row_iterator_next(it));

    var cells: RowCells = null;
    try testing.expectEqual(Result.success, row_cells_new(
        &lib.alloc.test_allocator,
        &cells,
    ));
    defer row_cells_free(cells);

    try testing.expectEqual(Result.success, row_get(it, .cells, @ptrCast(&cells)));
    try testing.expect(row_cells_next(cells));

    var bg: colorpkg.RGB.C = undefined;
    try testing.expectEqual(Result.success, row_cells_get(cells, .bg_color, @ptrCast(&bg)));
    try testing.expectEqual(@as(u8, 10), bg.r);
    try testing.expectEqual(@as(u8, 20), bg.g);
    try testing.expectEqual(@as(u8, 30), bg.b);
}

test "render: row cells fg_color no foreground" {
    var terminal: terminal_c.Terminal = null;
    try testing.expectEqual(Result.success, terminal_c.new(
        &lib.alloc.test_allocator,
        &terminal,
        .{
            .cols = 80,
            .rows = 24,
            .max_scrollback = 10_000,
        },
    ));
    defer terminal_c.free(terminal);

    // Write plain text (no foreground color set).
    terminal_c.vt_write(terminal, "hello", 5);

    var state: RenderState = null;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &state,
    ));
    defer free(state);

    try testing.expectEqual(Result.success, update(state, terminal));

    var it: RowIterator = null;
    try testing.expectEqual(Result.success, row_iterator_new(
        &lib.alloc.test_allocator,
        &it,
    ));
    defer row_iterator_free(it);

    try testing.expectEqual(Result.success, get(state, .row_iterator, @ptrCast(&it)));
    try testing.expect(row_iterator_next(it));

    var cells: RowCells = null;
    try testing.expectEqual(Result.success, row_cells_new(
        &lib.alloc.test_allocator,
        &cells,
    ));
    defer row_cells_free(cells);

    try testing.expectEqual(Result.success, row_get(it, .cells, @ptrCast(&cells)));
    try testing.expect(row_cells_next(cells));

    // No foreground set, should return invalid_value.
    var fg: colorpkg.RGB.C = undefined;
    try testing.expectEqual(Result.invalid_value, row_cells_get(cells, .fg_color, @ptrCast(&fg)));
}

test "render: row cells fg_color from style" {
    var terminal: terminal_c.Terminal = null;
    try testing.expectEqual(Result.success, terminal_c.new(
        &lib.alloc.test_allocator,
        &terminal,
        .{
            .cols = 80,
            .rows = 24,
            .max_scrollback = 10_000,
        },
    ));
    defer terminal_c.free(terminal);

    // Set an RGB foreground via SGR 38;2;R;G;B and write text.
    terminal_c.vt_write(terminal, "\x1b[38;2;10;20;30mA", 18);

    var state: RenderState = null;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &state,
    ));
    defer free(state);

    try testing.expectEqual(Result.success, update(state, terminal));

    var it: RowIterator = null;
    try testing.expectEqual(Result.success, row_iterator_new(
        &lib.alloc.test_allocator,
        &it,
    ));
    defer row_iterator_free(it);

    try testing.expectEqual(Result.success, get(state, .row_iterator, @ptrCast(&it)));
    try testing.expect(row_iterator_next(it));

    var cells: RowCells = null;
    try testing.expectEqual(Result.success, row_cells_new(
        &lib.alloc.test_allocator,
        &cells,
    ));
    defer row_cells_free(cells);

    try testing.expectEqual(Result.success, row_get(it, .cells, @ptrCast(&cells)));
    try testing.expect(row_cells_next(cells));

    var fg: colorpkg.RGB.C = undefined;
    try testing.expectEqual(Result.success, row_cells_get(cells, .fg_color, @ptrCast(&fg)));
    try testing.expectEqual(@as(u8, 10), fg.r);
    try testing.expectEqual(@as(u8, 20), fg.g);
    try testing.expectEqual(@as(u8, 30), fg.b);
}

test "render: colors get supports truncated sized struct" {
    var terminal: terminal_c.Terminal = null;
    try testing.expectEqual(Result.success, terminal_c.new(
        &lib.alloc.test_allocator,
        &terminal,
        .{
            .cols = 80,
            .rows = 24,
            .max_scrollback = 10_000,
        },
    ));
    defer terminal_c.free(terminal);

    var state: RenderState = null;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &state,
    ));
    defer free(state);

    try testing.expectEqual(Result.success, update(state, terminal));

    var colors: Colors = std.mem.zeroes(Colors);
    const sentinel: colorpkg.RGB.C = .{ .r = 0xAA, .g = 0xBB, .b = 0xCC };
    for (&colors.palette) |*entry| entry.* = sentinel;

    colors.size = @offsetOf(Colors, "palette") + @sizeOf(colorpkg.RGB.C) * 2;
    try testing.expectEqual(Result.success, colors_get(state, &colors));

    const state_colors = &state.?.state.colors;
    try testing.expectEqual(state_colors.palette[0].cval(), colors.palette[0]);
    try testing.expectEqual(state_colors.palette[1].cval(), colors.palette[1]);
    try testing.expectEqual(sentinel, colors.palette[2]);
}

test "render: get_multi success" {
    var terminal: terminal_c.Terminal = null;
    try testing.expectEqual(Result.success, terminal_c.new(
        &lib.alloc.test_allocator,
        &terminal,
        .{ .cols = 80, .rows = 24, .max_scrollback = 10_000 },
    ));
    defer terminal_c.free(terminal);

    var state: RenderState = null;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &state,
    ));
    defer free(state);

    try testing.expectEqual(Result.success, update(state, terminal));

    var cols: u16 = 0;
    var rows: u16 = 0;
    var written: usize = 0;

    const keys = [_]Data{ .cols, .rows };
    var values = [_]?*anyopaque{ @ptrCast(&cols), @ptrCast(&rows) };
    try testing.expectEqual(Result.success, get_multi(state, keys.len, &keys, &values, &written));
    try testing.expectEqual(keys.len, written);
    try testing.expectEqual(80, cols);
    try testing.expectEqual(24, rows);
}

test "render: get_multi null returns invalid_value" {
    var cols: u16 = 0;
    var values = [_]?*anyopaque{@ptrCast(&cols)};
    try testing.expectEqual(Result.invalid_value, get_multi(null, 1, null, &values, null));
}

test "render: get_multi colors" {
    var state: RenderState = null;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &state,
    ));
    defer free(state);

    var background: colorpkg.RGB.C = undefined;
    var foreground: colorpkg.RGB.C = undefined;
    var cursor_has_value = true;
    var written: usize = 0;

    const keys = [_]Data{ .color_background, .color_foreground, .color_cursor_has_value };
    var values = [_]?*anyopaque{
        @ptrCast(&background),
        @ptrCast(&foreground),
        @ptrCast(&cursor_has_value),
    };

    try testing.expectEqual(Result.success, get_multi(state, keys.len, &keys, &values, &written));
    try testing.expectEqual(keys.len, written);
    try testing.expectEqualDeep(colorpkg.RGB.C{ .r = 0, .g = 0, .b = 0 }, background);
    try testing.expectEqualDeep(colorpkg.RGB.C{ .r = 0xff, .g = 0xff, .b = 0xff }, foreground);
    try testing.expect(!cursor_has_value);
}

test "render: row_get_multi success" {
    var terminal: terminal_c.Terminal = null;
    try testing.expectEqual(Result.success, terminal_c.new(
        &lib.alloc.test_allocator,
        &terminal,
        .{ .cols = 80, .rows = 24, .max_scrollback = 10_000 },
    ));
    defer terminal_c.free(terminal);

    var state: RenderState = null;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &state,
    ));
    defer free(state);

    try testing.expectEqual(Result.success, update(state, terminal));

    var it: RowIterator = null;
    try testing.expectEqual(Result.success, row_iterator_new(
        &lib.alloc.test_allocator,
        &it,
    ));
    defer row_iterator_free(it);

    try testing.expectEqual(Result.success, get(state, .row_iterator, @ptrCast(&it)));
    try testing.expect(row_iterator_next(it));

    var dirty: bool = true;
    var written: usize = 0;

    const keys = [_]RowData{.dirty};
    var values = [_]?*anyopaque{@ptrCast(&dirty)};
    try testing.expectEqual(Result.success, row_get_multi(it, keys.len, &keys, &values, &written));
    try testing.expectEqual(keys.len, written);
}

test "render: row_get_multi null returns invalid_value" {
    var dirty: bool = false;
    var values = [_]?*anyopaque{@ptrCast(&dirty)};
    try testing.expectEqual(Result.invalid_value, row_get_multi(null, 1, null, &values, null));
}

test "render: row_cells_get_multi success" {
    var terminal: terminal_c.Terminal = null;
    try testing.expectEqual(Result.success, terminal_c.new(
        &lib.alloc.test_allocator,
        &terminal,
        .{ .cols = 80, .rows = 24, .max_scrollback = 10_000 },
    ));
    defer terminal_c.free(terminal);

    terminal_c.vt_write(terminal, "A", 1);

    var state: RenderState = null;
    try testing.expectEqual(Result.success, new(
        &lib.alloc.test_allocator,
        &state,
    ));
    defer free(state);

    try testing.expectEqual(Result.success, update(state, terminal));

    var it: RowIterator = null;
    try testing.expectEqual(Result.success, row_iterator_new(
        &lib.alloc.test_allocator,
        &it,
    ));
    defer row_iterator_free(it);

    try testing.expectEqual(Result.success, get(state, .row_iterator, @ptrCast(&it)));
    try testing.expect(row_iterator_next(it));

    var cells: RowCells = null;
    try testing.expectEqual(Result.success, row_cells_new(
        &lib.alloc.test_allocator,
        &cells,
    ));
    defer row_cells_free(cells);

    try testing.expectEqual(Result.success, row_get(it, .cells, @ptrCast(&cells)));
    try testing.expect(row_cells_next(cells));

    var raw: row.CRow = undefined;
    var written: usize = 0;

    const keys = [_]RowCellsData{.raw};
    var values = [_]?*anyopaque{@ptrCast(&raw)};
    try testing.expectEqual(Result.success, row_cells_get_multi(cells, keys.len, &keys, &values, &written));
    try testing.expectEqual(keys.len, written);
}

test "render: row_cells_get_multi null returns invalid_value" {
    var raw: row.CRow = undefined;
    var values = [_]?*anyopaque{@ptrCast(&raw)};
    try testing.expectEqual(Result.invalid_value, row_cells_get_multi(null, 1, null, &values, null));
}
