//! Universal SysEx parsing. These are the only messages specified in the
//! MIDI spec itself, rather than by a manufacturer.
//!
//! The main reference here was the _MIDI 1.0 Detailed Specification_.

use super::ManufacturerId;
use crate::midi::format_bytes;
use std::fmt::{Display, Formatter, Result as FmtResult};

pub const MF_ID_ROLAND: ManufacturerId = 0x41;

pub type DeviceId = u8;
/// "All call" is the name in the MIDI 1.0 Detailed Specification, but it is
/// more intuitive to call this the "broadcast" ID. That's what Roland do.
pub const DV_ID_BROADCAST: ManufacturerId = 0x7F;

pub type SubId1 = u8;
pub type SubId2 = u8;

#[derive(Debug)]
pub struct ParsedUniversalSysExBody<'a> {
    pub real_time: bool,
    pub device_id: DeviceId,
    pub sub_id1: SubId1,
    pub sub_id2: SubId2,
    pub data: &'a [u8],
}
impl Display for ParsedUniversalSysExBody<'_> {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        let &ParsedUniversalSysExBody {
            real_time: _,
            device_id,
            sub_id1,
            sub_id2,
            data,
        } = self;

        if device_id == DV_ID_BROADCAST {
            write!(f, "Broadcast")?;
        } else {
            write!(f, "Device {:02X}h", device_id)?;
        }
        write!(f, ", Sub-ID#1 {:02X}h", sub_id1)?;
        write!(f, ", Sub-ID#2 {:02X}h", sub_id2)?;
        write!(f, ": {}", format_bytes(data))?;
        Ok(())
    }
}

#[allow(clippy::result_unit_err)] // not much explanation can be given really
pub fn parse_sysex_body(real_time: bool, body: &[u8]) -> Result<ParsedUniversalSysExBody, ()> {
    let &[device_id, sub_id1, sub_id2, ref data @ ..] = body else {
        return Err(());
    };

    Ok(ParsedUniversalSysExBody {
        real_time,
        device_id,
        sub_id1,
        sub_id2,
        data,
    })
}
