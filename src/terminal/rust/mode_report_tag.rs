use crate::constants::*;

pub(crate) struct ModeReportTag {
    pub(crate) value: u64,
    pub(crate) ansi: bool,
}

pub(crate) fn mode_report_tag(tag: u16) -> ModeReportTag {
    ModeReportTag {
        value: u64::from(tag & MODE_VALUE_MASK),
        ansi: (tag & MODE_ANSI_MASK) != 0,
    }
}
