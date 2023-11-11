//! Roland SysEx parsing.
//!
//! My first reference for this is the Roland SC-7 owner's manual, and I have
//! also looked at the SC-55 and SC-55mkII owner's manuals. The same basic
//! protocol is used by all of these devices AFAICT, the differences are just in
//! the device IDs and "parameter maps". Presumably the rest of the Sound Canvas
//! series use this too. I don't know about other Roland devices.

use crate::midi::format_bytes;
use std::fmt::{Display, Formatter, Result as FmtResult};

use super::{DeviceId, ManufacturerId};

pub const MF_ID_ROLAND: ManufacturerId = 0x41;

pub type ModelId = u8;

/// Roland SC-7, according to the SC-7 owner's manual. This is used to control
/// its effects, it uses the GS ID for a handful of things.
pub const MD_ID_ROLAND_SC_7: DeviceId = 0x56;
/// Roland GS, according to the SC-55mkII owner's manual.
pub const MD_ID_ROLAND_GS: DeviceId = 0x42;
/// Roland SC-55 and SC-155 device ID, according to the SC-55mkII owner's
/// manual.
pub const MD_ID_ROLAND_SC_55: DeviceId = 0x45;

pub type CommandId = u8;

#[derive(Debug)]
pub enum ParsedRolandSysExBody<'a> {
    /// Roland SC-7 manual says "Roland's MIDI implementation uses the following
    /// data format for all Exclusive messages" and refers to it as "Type IV".
    /// You can see similar text in many other Roland product manuals, including
    /// the SC-55 for example. I don't know where this numbering comes from.
    TypeIV {
        model_id: ModelId,
        command_id: CommandId,
        body: &'a [u8],
    },
}
impl Display for ParsedRolandSysExBody<'_> {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            &ParsedRolandSysExBody::TypeIV {
                model_id,
                command_id,
                body,
            } => {
                match model_id {
                    MD_ID_ROLAND_SC_7 => write!(f, "Roland SC-7")?,
                    MD_ID_ROLAND_SC_55 => write!(f, "Roland SC-55/SC-155")?,
                    MD_ID_ROLAND_GS => write!(f, "Roland GS")?,
                    _ => write!(f, "Model {:02X}h", model_id)?,
                }
                write!(f, ", Command {:02X}h", command_id)?;
                write!(f, ": ")?;
                write!(f, "{}", format_bytes(body))?;
            }
        }
        Ok(())
    }
}

#[allow(clippy::result_unit_err)] // not much explanation can be given really
pub fn parse_sysex_body(body: &[u8]) -> Result<ParsedRolandSysExBody, ()> {
    let &[model_id, command_id, ref body @ ..] = body else {
        return Err(());
    };

    Ok(ParsedRolandSysExBody::TypeIV {
        model_id,
        command_id,
        body,
    })
}
