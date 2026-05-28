const std = @import("std");
const lib = @import("../lib.zig");
const CAllocator = lib.alloc.Allocator;
const colorpkg = @import("../color.zig");
const page = @import("../page.zig");
const size = @import("../size.zig");
const PageList = @import("../PageList.zig");
const Style = @import("../style.zig").Style;
const sgr = @import("../sgr.zig");
const render = @import("render.zig");
const renderpkg = @import("../render.zig");
const style_c = @import("style.zig");

fn colorFromC(c: style_c.Color) Style.Color {
    return switch (c.tag) {
        .none => .none,
        .palette => .{ .palette = c.value.palette },
        .rgb => .{ .rgb = colorpkg.RGB.fromC(c.value.rgb) },
    };
}

fn underlineFromC(v: c_int) sgr.Attribute.Underline {
    return switch (v) {
        1 => .single,
        2 => .double,
        3 => .curly,
        4 => .dotted,
        5 => .dashed,
        else => .none,
    };
}

fn styleFromC(s: *const style_c.Style) Style {
    return .{
        .fg_color = colorFromC(s.fg_color),
        .bg_color = colorFromC(s.bg_color),
        .underline_color = colorFromC(s.underline_color),
        .flags = .{
            .bold = s.bold,
            .italic = s.italic,
            .faint = s.faint,
            .blink = s.blink,
            .inverse = s.inverse,
            .invisible = s.invisible,
            .strikethrough = s.strikethrough,
            .overline = s.overline,
            .underline = underlineFromC(s.underline),
        },
    };
}

pub fn renderOwnedBegin(
    state: *render.RenderStateWrapper,
    alloc_: ?*const CAllocator,
    rows: size.CellCountInt,
    cols: size.CellCountInt,
    screen_key: @import("../ScreenSet.zig").Key,
    dirty: c_int,
    cursor_visual_style: u8,
    cursor_visible: bool,
    cursor_blinking: bool,
    cursor_password_input: bool,
    cursor_active_x: size.CellCountInt,
    cursor_active_y: size.CellCountInt,
    cursor_style: *const Style,
    cursor_cell: *const page.Cell,
    viewport_pin: *const PageList.Pin,
    background: colorpkg.RGB.C,
    foreground: colorpkg.RGB.C,
    cursor_present: bool,
    cursor: colorpkg.RGB.C,
    palette: *const colorpkg.PaletteC,
) callconv(lib.calling_conv) c_int {
    const alloc = lib.alloc.default(alloc_);
    var rs = &state.state;

    rs.rows = rows;
    rs.cols = cols;
    rs.screen = screen_key;
    rs.dirty = @enumFromInt(dirty);
    rs.viewport_pin = viewport_pin.*;
    rs.selection_cache = null;

    rs.cursor.active = .{ .x = cursor_active_x, .y = cursor_active_y };
    rs.cursor.style = cursor_style.*;
    rs.cursor.cell = cursor_cell.*;
    rs.cursor.visual_style = @enumFromInt(cursor_visual_style);
    rs.cursor.visible = cursor_visible;
    rs.cursor.blinking = cursor_blinking;
    rs.cursor.password_input = cursor_password_input;
    rs.cursor.viewport = null;

    rs.colors.background = colorpkg.RGB.fromC(background);
    rs.colors.foreground = colorpkg.RGB.fromC(foreground);
    rs.colors.cursor = if (cursor_present) colorpkg.RGB.fromC(cursor) else null;
    inline for (0..256) |i| {
        rs.colors.palette[i] = colorpkg.RGB.fromC(palette[i]);
    }

    if (rs.row_data.len != rows) {
        if (rs.row_data.len < rows) {
            const old_len = rs.row_data.len;
            rs.row_data.resize(alloc, rows) catch return -1;
            var row_data = rs.row_data.slice();
            for (old_len..rows) |y| {
                row_data.set(y, .{
                    .arena = .{},
                    .pin = undefined,
                    .raw = undefined,
                    .cells = .empty,
                    .dirty = true,
                    .selection = null,
                    .highlights = .empty,
                });
            }
        } else {
            const row_data = rs.row_data.slice();
            for (
                row_data.items(.arena)[rows..],
                row_data.items(.cells)[rows..],
            ) |arena_state, *cells| {
                var arena: std.heap.ArenaAllocator = arena_state.promote(alloc);
                arena.deinit();
                cells.deinit(alloc);
            }
            rs.row_data.shrinkRetainingCapacity(rows);
        }
    }

    return 0;
}

pub fn renderOwnedRow(
    state: *render.RenderStateWrapper,
    alloc_: ?*const CAllocator,
    y: size.CellCountInt,
    pin: *const PageList.Pin,
    row: *const page.Row,
    cells: [*]const page.Cell,
    cols: size.CellCountInt,
    dirty: bool,
    selection_present: bool,
    selection_start: size.CellCountInt,
    selection_end: size.CellCountInt,
    is_cursor_row: bool,
    cursor_x: size.CellCountInt,
    cursor_wide_tail: bool,
) callconv(lib.calling_conv) c_int {
    const alloc = lib.alloc.default(alloc_);
    const rs = &state.state;
    const row_data = rs.row_data.slice();

    row_data.items(.pin)[y] = pin.*;
    row_data.items(.raw)[y] = row.*;
    row_data.items(.dirty)[y] = dirty;
    row_data.items(.selection)[y] = if (selection_present)
        .{ selection_start, selection_end }
    else
        null;

    if (is_cursor_row) {
        rs.cursor.viewport = .{
            .x = cursor_x,
            .y = y,
            .wide_tail = cursor_wide_tail,
        };
    }

    const cell_list = &row_data.items(.cells)[y];
    cell_list.resize(alloc, cols) catch return -1;
    const cells_slice = cell_list.slice();
    @memcpy(cells_slice.items(.raw), cells[0..cols]);

    if (!row.managedMemory()) return 0;

    const p: *page.Page = @ptrCast(@alignCast(&pin.node.data));
    const page_cells = cells[0..cols];
    const cells_grapheme = cells_slice.items(.grapheme);

    for (page_cells, 0..) |*page_cell, x| {
        switch (page_cell.content_tag) {
            .codepoint, .bg_color_rgb, .bg_color_palette => {},
            .codepoint_grapheme => {
                const grapheme = p.lookupGrapheme(page_cell) orelse &.{};
                var arena = row_data.items(.arena)[y].promote(alloc);
                defer row_data.items(.arena)[y] = arena.state;
                cells_grapheme[x] = arena.allocator().dupe(u21, grapheme) catch return -1;
            },
        }

        if (page_cell.content_tag == .bg_color_rgb) {
            cells_slice.items(.style)[x] = .{ .bg_color = .{ .rgb = .{
                .r = page_cell.content.color_rgb.r,
                .g = page_cell.content.color_rgb.g,
                .b = page_cell.content.color_rgb.b,
            } } };
        } else if (page_cell.content_tag == .bg_color_palette) {
            cells_slice.items(.style)[x] = .{ .bg_color = .{
                .palette = page_cell.content.color_palette,
            } };
        }
    }

    return 0;
}

pub fn renderOwnedCellStyle(
    state: *render.RenderStateWrapper,
    y: size.CellCountInt,
    x: size.CellCountInt,
    style: *const style_c.Style,
) callconv(lib.calling_conv) c_int {
    const rs = &state.state;
    if (y >= rs.row_data.len) return -2;
    const row_cells = &rs.row_data.items(.cells)[y];
    if (x >= row_cells.len) return -2;
    row_cells.slice().items(.style)[x] = styleFromC(style);
    return 0;
}

pub fn renderOwnedEnd(
    state: *render.RenderStateWrapper,
    any_dirty: bool,
) callconv(lib.calling_conv) c_int {
    const rs = &state.state;
    if (any_dirty and rs.dirty == .false) {
        rs.dirty = .partial;
    }
    return 0;
}
