//! MIDI System Exclusive message (SysEx) parser and builder.
//!
//! SysExes are an extensibility feature of the MIDI standard and almost always
//! vendor-specific, so a fully general parser is not possible. This code only
//! attempts to parse a few formats it knows about, and for the rest it gives
//! back a generic "unknown" kind. Likewise, the building here only works for
//! known formats. Manufacturer ID-specific parsing is delegated to child
//! modules.
//!
//! The main reference here was the _MIDI 1.0 Detailed Specification_.

pub mod roland;
pub mod universal;

use crate::midi::format_bytes;
use crate::ui::{Menu, MenuItemResult};
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
    pub manufacturer_id: ManufacturerId,
    pub content: MaybeParsed<'a, ParsedSysExBody<'a>>,
}
impl Display for ParsedSysEx<'_> {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self.manufacturer_id {
            MF_ID_ROLAND => write!(f, "Roland")?,
            MF_ID_UNIVERSAL_NON_REAL_TIME => write!(f, "Universal Non-Real Time")?,
            MF_ID_UNIVERSAL_REAL_TIME => write!(f, "Universal Real Time")?,
            other => write!(f, "Manufacturer {:02X}h", other)?,
        }
        write!(f, ": {}", self.content)?;
        Ok(())
    }
}

/// Generate a SysEx message or subcomponent of a SysEx message (depending on
/// the implementing type; use [ParsedSysEx] for a full SysEx).
pub trait SysExGenerator: std::fmt::Debug {
    /// Write the message/subcomponent to `out`. If this is [ParsedSysEx], it
    /// must write a complete SysEx message (including initial `F0h` and ending
    /// `F7h`) to `out`. Other implementations must be careful not to duplicate
    /// data that would be output by the type for the containing
    /// message/subcomponent, and not to omit anything needed for this
    /// subcomponent.
    fn generate(&self, out: &mut Vec<u8>);
}

/// Contains a parsed version of something, if it was understood, or otherwise
/// the unparsed form, if it wasn't.
#[derive(Debug)]
pub enum MaybeParsed<'a, T> {
    Parsed(T),
    Unknown(&'a [u8]),
}
impl<T> Display for MaybeParsed<'_, T>
where
    T: Display,
{
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            MaybeParsed::Parsed(parsed) => write!(f, "{}", parsed),
            MaybeParsed::Unknown(bytes) => write!(f, "(unknown) {}", format_bytes(bytes)),
        }
    }
}
impl<T> SysExGenerator for MaybeParsed<'_, T>
where
    T: SysExGenerator,
{
    fn generate(&self, out: &mut Vec<u8>) {
        match self {
            MaybeParsed::Parsed(parsed) => parsed.generate(out),
            MaybeParsed::Unknown(bytes) => out.extend_from_slice(bytes),
        }
    }
}

#[derive(Debug)]
pub enum ParsedSysExBody<'a> {
    Roland(roland::ParsedRolandSysExBody<'a>),
    Universal(universal::ParsedUniversalSysExBody<'a>),
}
impl Display for ParsedSysExBody<'_> {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            ParsedSysExBody::Roland(parsed) => write!(f, "{}", parsed),
            ParsedSysExBody::Universal(parsed) => write!(f, "{}", parsed),
        }
    }
}
impl SysExGenerator for ParsedSysExBody<'_> {
    fn generate(&self, out: &mut Vec<u8>) {
        match self {
            ParsedSysExBody::Roland(parsed) => parsed.generate(out),
            ParsedSysExBody::Universal(_) => todo!(),
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

    let &[manufacturer_id, ref data @ ..] = data else {
        return Err(ParseFailure::IncompleteSysEx);
    };

    let content = match (manufacturer_id, data) {
        (MF_ID_ROLAND, body) => roland::parse_sysex_body(body).map(ParsedSysExBody::Roland),
        (MF_ID_UNIVERSAL_NON_REAL_TIME, body) => {
            universal::parse_sysex_body(/* real_time: */ false, body)
                .map(ParsedSysExBody::Universal)
        }
        (MF_ID_UNIVERSAL_REAL_TIME, body) => {
            universal::parse_sysex_body(/* real_time: */ true, body).map(ParsedSysExBody::Universal)
        }
        _ => Err(()),
    }
    .map_or(MaybeParsed::Unknown(data), |parsed| {
        MaybeParsed::Parsed(parsed)
    });

    Ok(ParsedSysEx {
        manufacturer_id,
        content,
    })
}

impl SysExGenerator for ParsedSysEx<'_> {
    fn generate(&self, out: &mut Vec<u8>) {
        out.push(0xF0);
        out.push(self.manufacturer_id);
        self.content.generate(out);
        out.push(0xF7);
    }
}

/// Convenience implementation of [SysExGenerator] for constant SysExes strings.
#[derive(Debug)]
pub struct StaticSysExGenerator(pub &'static [u8]);
impl SysExGenerator for StaticSysExGenerator {
    fn generate(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(self.0);
    }
}

type SysExGeneratorMenuTrait = dyn Menu<Box<dyn SysExGenerator>>;

/// Provides a menu for generating a SysEx.
pub fn generate_sysex() -> impl Menu<Box<dyn SysExGenerator>> {
    struct SysExGeneratorMenu;

    #[allow(clippy::type_complexity)]
    const SYSEX_GENERATORS: &[(&str, fn() -> Box<SysExGeneratorMenuTrait>)] = &[
        // The universal SysExes are at the top of the list because they're not
        // vendor-specific, but their numbering ought to place them last.
        // Putting the manufacturer ID at the end avoids breaking the sorting
        // that typing hex directly into a <select> relies on.
        (
            "Universal Non-Real Time (7Eh)",
            universal::generate_nrt_sysex,
        ),
        ("41h — Roland", roland::generate_sysex),
    ];

    impl Menu<Box<dyn SysExGenerator>> for SysExGeneratorMenu {
        fn items_count(&self) -> usize {
            SYSEX_GENERATORS.len()
        }
        fn item_label(&self, item_idx: usize, write_to: &mut dyn std::fmt::Write) -> FmtResult {
            write!(write_to, "{}", SYSEX_GENERATORS[item_idx].0)
        }
        fn item_descend(&self, item_idx: usize) -> MenuItemResult<Box<dyn SysExGenerator>> {
            MenuItemResult::Submenu(SYSEX_GENERATORS[item_idx].1())
        }
    }

    SysExGeneratorMenu
}
