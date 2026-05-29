use crate::constants::*;
use crate::early::*;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DsrColorScheme {
    Light = 0,
    Dark = 1,
}

impl DsrColorScheme {
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0 => Some(DsrColorScheme::Light),
            1 => Some(DsrColorScheme::Dark),
            _ => None,
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceStatusRequest {
    OperatingStatus = 0,
    CursorPosition = 1,
    ColorScheme = 2,
}

impl Default for DeviceStatusRequest {
    fn default() -> Self {
        DeviceStatusRequest::OperatingStatus
    }
}

impl DeviceStatusRequest {
    pub fn from_int(value: u16, question: bool) -> Option<Self> {
        match (value, question) {
            (5, false) => Some(DeviceStatusRequest::OperatingStatus),
            (6, false) => Some(DeviceStatusRequest::CursorPosition),
            (996, true) => Some(DeviceStatusRequest::ColorScheme),
            _ => None,
        }
    }
}
