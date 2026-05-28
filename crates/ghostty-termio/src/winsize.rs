//! Terminal dimensions for a PTY (`src/pty.zig` `winsize`).

/// Row/column and pixel size of a pseudo-terminal.
///
/// Defaults match Zig `pty.winsize` (reasonable screen size when unset).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Winsize {
    pub rows: u16,
    pub cols: u16,
    pub x_pixels: u16,
    pub y_pixels: u16,
}

impl Default for Winsize {
    fn default() -> Self {
        Self {
            rows: 100,
            cols: 80,
            x_pixels: 800,
            y_pixels: 600,
        }
    }
}

#[cfg(unix)]
impl Winsize {
    pub(crate) fn to_libc(self) -> libc::winsize {
        libc::winsize {
            ws_row: self.rows,
            ws_col: self.cols,
            ws_xpixel: self.x_pixels,
            ws_ypixel: self.y_pixels,
        }
    }

    pub(crate) fn from_libc(ws: libc::winsize) -> Self {
        Self {
            rows: ws.ws_row,
            cols: ws.ws_col,
            x_pixels: ws.ws_xpixel,
            y_pixels: ws.ws_ypixel,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_matches_zig() {
        let ws = Winsize::default();
        assert_eq!(ws.rows, 100);
        assert_eq!(ws.cols, 80);
        assert_eq!(ws.x_pixels, 800);
        assert_eq!(ws.y_pixels, 600);
    }
}
