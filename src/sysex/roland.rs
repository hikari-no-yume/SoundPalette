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

/// Roland SC-7, according to the SC-7 owner's manual. This device also uses
/// [MD_ID_ROLAND_GS].
pub const MD_ID_ROLAND_SC_7: DeviceId = 0x56;
/// Roland GS, according to the SC-55mkII owner's manual.
pub const MD_ID_ROLAND_GS: DeviceId = 0x42;
/// Roland SC-55 and SC-155 device ID, according to the SC-55mkII owner's
/// manual. This device also uses [MD_ID_ROLAND_GS].
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
        command: match parse_sysex_command(model_id, command_id, body) {
            Ok(parsed) => MaybeParsed::Parsed(parsed),
            Err(()) => MaybeParsed::Unknown(body),
        },
    })
}

#[derive(Debug)]
pub enum ParsedRolandSysExCommand<'a> {
    /// "Data set 1" aka "DT1". The `address` and `data` are the raw parsing
    /// results, whereas the other fields are interpretation.
    DT1 {
        address: &'a [u8],
        data: &'a [u8],
        /// Was the checksum correct? Wrong checksums are tolerated because this
        /// is more helpful in MIDI debugging than displaying no info.
        valid_checksum: bool,
        /// Name of the parameter block the address seems to be for, if it could
        /// be found.
        block_name: Option<&'static str>,
        /// Information about the parameter the address seems to be for, if it
        /// could be found.
        param_info: Option<&'static Parameter>,
        /// If parameter information could be found, this is whether the
        /// size of the data matches the parameter. This error is tolerated for
        /// the same reason as invalid checksums. If parameter information could
        /// not be found, this value is not meaningful.
        invalid_size: bool,
    },
}
impl Display for ParsedRolandSysExCommand<'_> {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            &ParsedRolandSysExCommand::DT1 {
                address,
                data,
                valid_checksum,
                block_name,
                param_info,
                invalid_size,
            } => {
                write!(f, "Data set 1: ")?;

                assert!(address.len() == 3); // TODO: refactor?
                if let Some(block_name) = block_name {
                    write!(f, "{} § ", block_name)?;
                    if let Some(param_info) = param_info {
                        write!(
                            f,
                            "{}{}",
                            param_info.name,
                            if invalid_size { " (WRONG SIZE)" } else { "" }
                        )?;
                    } else {
                        write!(f, "(unknown) {}", format_bytes(&address[2..]))?;
                    }
                } else {
                    assert!(param_info.is_none());
                    assert!(!invalid_size);
                    write!(f, "(unknown) {}", format_bytes(address))?;
                }

                write!(
                    f,
                    " => {}{}",
                    format_bytes(data),
                    if valid_checksum {
                        ""
                    } else {
                        " (WRONG CHECKSUM)"
                    }
                )
            }
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
    model_id: ModelId,
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

            let (block_name, param_info) = look_up_parameter(model_id, address);

            let invalid_size = param_info.map_or(false, |param| param.size as usize != data.len());

            Ok(ParsedRolandSysExCommand::DT1 {
                address,
                data,
                valid_checksum,
                block_name,
                param_info,
                invalid_size,
            })
        }
        _ => Err(()),
    }
}

/// Uses [ADDRESS_BLOCK_MAP_MAP] to look up the name of the address block and
/// the details of the parameter using an address, if possible.
pub fn look_up_parameter(
    model_id: ModelId,
    address: &[u8],
) -> (Option<&'static str>, Option<&'static Parameter>) {
    let &[msb, smsb, lsb] = address else {
        return (None, None);
    };

    let Some(&(_, abm)) = ADDRESS_BLOCK_MAP_MAP
        .iter()
        .find(|&&(model_id2, _)| model_id == model_id2)
    else {
        return (None, None);
    };

    let Some(&(_, _, block_name, pam)) = abm
        .iter()
        .find(|&&(msb2, smsb2, _, _)| (msb, smsb) == (msb2, smsb2))
    else {
        return (None, None);
    };

    (
        Some(block_name),
        match pam.iter().find(|&&(lsb2, _)| lsb == lsb2) {
            Some((_, param)) => Some(param),
            None => None,
        },
    )
}

/// List of "Address Block Maps" by model ID, to facilitate automated parameter
/// lookup.
pub type AddressBlockMapMap = &'static [(ModelId, AddressBlockMap)];

/// "Address Block Map" in the style of the Roland SC-7 owner's manual.
/// Describes the high-level layout of the parameter map (the first two bytes of
/// the address, which are the most and second-most significant bytes,
/// respectively). Each block has a human-readable name.
pub type AddressBlockMap = &'static [(u8, u8, &'static str, ParameterAddressMap)];

/// "Parameter Block Map" in the style of the Roland SC-7 owner's manual.
/// Describes the low-level layout of the parameter map (the last byte of the
/// address, which is the least significant byte). See also [AddressBlockMap].
pub type ParameterAddressMap = &'static [(u8, Parameter)];

/// The rows from a "Parameter Address Map" (see [ParameterAddressMap]).
#[derive(Debug)]
pub struct Parameter {
    /// "Size": Number of data bytes expected for this parameter
    pub size: u8,
    /// "Name": Human-readable name for this parameter
    pub name: &'static str,
    // TODO: handle Data, Description, Default Value etc in some reasonable way
}

// All the maps are in their own module to keep this one small.
mod maps;
pub use maps::ADDRESS_BLOCK_MAP_MAP;
