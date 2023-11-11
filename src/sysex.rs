//! MIDI System Exclusive message (SysEx) parser.
//!
//! SysExes are an extensibility feature of the MIDI standard and almost always
//! vendor-specific, so a fully general parser is not possible. This code only
//! attempts to parse a few formats it knows about, and for the rest it gives
//! back a generic "unknown" kind.

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

/// Contains a parsed version of something, if it was understood, or otherwise
/// the unparsed form, if it wasn't.
#[derive(Debug)]
pub enum MaybeParsed<'a, T> {
    Parsed(T),
    Unknown(&'a [u8]),
}

#[derive(Debug)]
pub enum ParsedSysExBody<'a> {
    /// Roland SC-7 manual says "Roland's MIDI implementation uses the following
    /// data format for all Exclusive messages" and refers to it as "Type IV".
    /// You can see similar text in many other Roland product manuals, including
    /// the SC-55 for example. I don't know where this numbering comes from.
    RolandTypeIV {
        model_id: RolandModelId,
        command_id: RolandCommandId,
        body: &'a [u8],
    },
}

pub type RolandModelId = u8;
pub type RolandCommandId = u8;

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
        (MF_ID_ROLAND, &[model_id, command_id, ref body @ ..]) => {
            MaybeParsed::Parsed(ParsedSysExBody::RolandTypeIV {
                model_id,
                command_id,
                body,
            })
        }
        _ => MaybeParsed::Unknown(data),
    };

    Ok(ParsedSysEx {
        manufacturer_id,
        device_id,
        content,
    })
}
