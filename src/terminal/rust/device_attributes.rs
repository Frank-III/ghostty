use crate::constants::*;
use crate::early::*;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DeviceAttributeReq {
    #[default]
    Primary = 0,
    Secondary = 1,
    Tertiary = 2,
}

impl DeviceAttributeReq {
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0 => Some(DeviceAttributeReq::Primary),
            1 => Some(DeviceAttributeReq::Secondary),
            2 => Some(DeviceAttributeReq::Tertiary),
            _ => None,
        }
    }
}
