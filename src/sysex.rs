//! MIDI System Exclusive message (SysEx) parser.
//!
//! SysExes are an extensibility feature of the MIDI standard and almost always
//! vendor-specific, so a fully general parser is not possible. This code only
//! attempts to parse a few formats it knows about, and for the rest it gives
//! back a generic "unknown" kind. Child modules handle manufacturer-specific
//! stuff.
//!
//! The main reference here was the _MIDI 1.0 Detailed Specification_.

pub mod roland;

use crate::midi::format_bytes;
use std::fmt::{Display, Formatter, Result as FmtResult};

#[derive(Debug)]
pub enum ParseFailure {
    NotSysEx,
    IncompleteSysEx,
}

pub type ManufacturerId = u8;
pub const MF_ID_ROLAND: ManufacturerId = 0x41;
pub const MF_ID_UNIVERSAL_NON_REAL_TIME: ManufacturerId = 0x7E;
pub const MF_ID_UNIVERSAL_REAL_TIME: ManufacturerId = 0x7F;

pub type DeviceId = u8;
/// "All call" is the name in the MIDI 1.0 Detailed Specification, but it might
/// be more intuitive to call this the "broadcast" ID.
pub const DV_ID_ALL_CALL: ManufacturerId = 0x7F;

#[derive(Debug)]
#[allow(dead_code)] // only used by Debug for now
pub struct ParsedSysEx<'a> {
    manufacturer_id: ManufacturerId,
    device_id: DeviceId,
    content: MaybeParsed<'a, ParsedSysExBody<'a>>,
}
impl Display for ParsedSysEx<'_> {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self.manufacturer_id {
            MF_ID_ROLAND => write!(f, "Roland")?,
            MF_ID_UNIVERSAL_NON_REAL_TIME => write!(f, "Universal Non-Real Time")?,
            MF_ID_UNIVERSAL_REAL_TIME => write!(f, "Universal Real Time")?,
            other => write!(f, "Manufacturer {:02X}h", other)?,
        }
        write!(f, ", Device {:02X}h", self.device_id)?;
        if self.device_id == DV_ID_ALL_CALL {
            write!(f, " (All Call)")?;
        }
        write!(f, ": ")?;
        match &self.content {
            MaybeParsed::Parsed(parsed) => write!(f, "{}", parsed)?,
            MaybeParsed::Unknown(bytes) => write!(f, "(unknown) {}", format_bytes(bytes))?,
        }
        Ok(())
    }
}

/// Contains a parsed version of something, if it was understood, or otherwise
/// the unparsed form, if it wasn't.
#[derive(Debug)]
pub enum MaybeParsed<'a, T> {
    Parsed(T),
    Unknown(&'a [u8]),
}

#[derive(Debug)]
pub enum ParsedSysExBody<'a> {
    Roland(roland::ParsedRolandSysExBody<'a>),
}
impl Display for ParsedSysExBody<'_> {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            ParsedSysExBody::Roland(parsed) => write!(f, "{}", parsed),
        }
    }
}

pub fn parse_sysex(data: &[u8]) -> Result<ParsedSysEx, ParseFailure> {
    // TODO: How to handle SysExes broken up across multiple messages?
    //       Probably the caller's responsibility?
    let &[0xF0, ref data @ ..] = data else {
        return Err(ParseFailure::NotSysEx);
    };
    let &[ref data @ .., 0xF7] = data else {
        return Err(ParseFailure::IncompleteSysEx);
    };

    assert!(!data.iter().any(|&byte| byte > 0x7F)); // TODO: return error?

    let &[manufacturer_id, device_id, ref data @ ..] = data else {
        return Err(ParseFailure::IncompleteSysEx);
    };

    let content = match (manufacturer_id, data) {
        (MF_ID_ROLAND, body) => roland::parse_sysex_body(body).map(ParsedSysExBody::Roland),
        _ => Err(()),
    }
    .map_or(MaybeParsed::Unknown(data), |parsed| {
        MaybeParsed::Parsed(parsed)
    });

    Ok(ParsedSysEx {
        manufacturer_id,
        device_id,
        content,
    })
}
