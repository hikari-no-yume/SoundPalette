//! Roland SysEx parsing.
//!
//! My first reference for this is the Roland SC-7 owner's manual, and I have
//! also looked at the SC-55 and SC-55mkII owner's manuals. The same basic
//! protocol is used by all of these devices AFAICT, the differences are just in
//! the device IDs and "parameter maps". Presumably the rest of the Sound Canvas
//! series use this too. I don't know about other Roland devices.

use super::{DeviceId, ManufacturerId, MaybeParsed};
use crate::midi::format_bytes;
use std::fmt::{Display, Formatter, Result as FmtResult};

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

/// "Data set 1" aka "DT1".
pub const CM_ID_DT1: CommandId = 0x12;

// TODO: support "Request data 1" aka "RQ1".

#[derive(Debug)]
pub enum ParsedRolandSysExBody<'a> {
    /// Roland SC-7 manual says "Roland's MIDI implementation uses the following
    /// data format for all Exclusive messages" and refers to it as "Type IV".
    /// You can see similar text in many other Roland product manuals, including
    /// the SC-55 for example. I don't know where this numbering comes from.
    TypeIV {
        model_id: ModelId,
        command_id: CommandId,
        command: MaybeParsed<'a, ParsedRolandSysExCommand<'a>>,
    },
}
impl Display for ParsedRolandSysExBody<'_> {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            &ParsedRolandSysExBody::TypeIV {
                model_id,
                command_id,
                ref command,
            } => {
                match model_id {
                    MD_ID_ROLAND_SC_7 => write!(f, "Roland SC-7")?,
                    MD_ID_ROLAND_SC_55 => write!(f, "Roland SC-55/SC-155")?,
                    MD_ID_ROLAND_GS => write!(f, "Roland GS")?,
                    _ => write!(f, "Model {:02X}h", model_id)?,
                }
                if let MaybeParsed::Unknown(_) = command {
                    write!(f, ", Command {:02X}h", command_id)?
                }
                write!(f, ": {}", command)?;
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
        command: match parse_sysex_command(command_id, body) {
            Ok(parsed) => MaybeParsed::Parsed(parsed),
            Err(()) => MaybeParsed::Unknown(body),
        },
    })
}

#[derive(Debug)]
pub enum ParsedRolandSysExCommand<'a> {
    DT1 {
        address: &'a [u8],
        data: &'a [u8],
        /// Wrong checksums are tolerated because this is more helpful in MIDI
        /// debugging than displaying no info.
        valid_checksum: bool,
    },
}
impl Display for ParsedRolandSysExCommand<'_> {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            &ParsedRolandSysExCommand::DT1 {
                address,
                data,
                valid_checksum,
            } => write!(
                f,
                "Data set 1: {} => {}{}",
                format_bytes(address),
                format_bytes(data),
                if valid_checksum {
                    ""
                } else {
                    " (WRONG CHECKSUM)"
                }
            ),
        }
    }
}

pub fn validate_checksum(data_including_checksum: &[u8]) -> bool {
    let mut sum: u8 = 0;
    for &byte in data_including_checksum {
        assert!(byte < 0x80);
        sum = (sum + byte) & 0x7F;
    }
    sum == 0
}

#[allow(clippy::result_unit_err)] // not much explanation can be given really
pub fn parse_sysex_command(
    command_id: CommandId,
    body: &[u8],
) -> Result<ParsedRolandSysExCommand, ()> {
    match (command_id, body) {
        // It's unclear from the sources I've used whether DT1 always takes a
        // 3-byte address, or whether this is only on certain models?
        // Certainly, the SC-7 and SC-55 use a 3-byte address.
        // The SC-55mkII manual remarks "SC-55mkII only recognizes the DT1
        // messages whose address and size match the Parameter Address Map"
        // which hints towards the latter, in my view. Maybe we can parameterise
        // this eventually.
        (CM_ID_DT1, &[_addr0, _addr1, _addr2, ref data @ .., _checksum]) => {
            let address = &body[..3];
            let valid_checksum = validate_checksum(body);
            Ok(ParsedRolandSysExCommand::DT1 {
                address,
                data,
                valid_checksum,
            })
        }
        _ => Err(()),
    }
}
