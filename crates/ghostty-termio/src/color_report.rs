//! OSC palette color report formatting (`stream_handler.zig` subset).

use ghostty_config::{OscColorReportFormat, RgbColor};

/// Format a palette color query response (OSC 4).
pub fn format_palette_color_report(
    format: OscColorReportFormat,
    index: u8,
    color: RgbColor,
) -> Option<String> {
    match format {
        OscColorReportFormat::None => None,
        OscColorReportFormat::SixteenBit => Some(format!(
            "\x1b]4;{index};rgb:{r:04x}/{g:04x}/{b:04x}\x1b\\",
            index = index,
            r = u16::from(color.r) * 257,
            g = u16::from(color.g) * 257,
            b = u16::from(color.b) * 257,
        )),
        OscColorReportFormat::EightBit => Some(format!(
            "\x1b]4;{index};rgb:{r:02x}/{g:02x}/{b:02x}\x1b\\",
            index = index,
            r = color.r,
            g = color.g,
            b = color.b,
        )),
    }
}

/// Format a dynamic color query response (OSC 10/11/12).
pub fn format_dynamic_color_report(
    format: OscColorReportFormat,
    osc_id: u8,
    color: RgbColor,
) -> Option<String> {
    match format {
        OscColorReportFormat::None => None,
        OscColorReportFormat::SixteenBit => Some(format!(
            "\x1b]{osc_id};rgb:{r:04x}/{g:04x}/{b:04x}\x1b\\",
            osc_id = osc_id,
            r = u16::from(color.r) * 257,
            g = u16::from(color.g) * 257,
            b = u16::from(color.b) * 257,
        )),
        OscColorReportFormat::EightBit => Some(format!(
            "\x1b]{osc_id};rgb:{r:02x}/{g:02x}/{b:02x}\x1b\\",
            osc_id = osc_id,
            r = color.r,
            g = color.g,
            b = color.b,
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn palette_report_16_bit() {
        let color = RgbColor { r: 1, g: 2, b: 3 };
        let msg = format_palette_color_report(OscColorReportFormat::SixteenBit, 4, color).unwrap();
        assert!(msg.starts_with("\x1b]4;4;rgb:"));
        assert!(msg.ends_with("\x1b\\"));
    }

    #[test]
    fn dynamic_report_8_bit() {
        let color = RgbColor {
            r: 0xff,
            g: 0x00,
            b: 0x80,
        };
        let msg = format_dynamic_color_report(OscColorReportFormat::EightBit, 10, color).unwrap();
        assert_eq!(msg, "\x1b]10;rgb:ff/00/80\x1b\\");
    }
}
