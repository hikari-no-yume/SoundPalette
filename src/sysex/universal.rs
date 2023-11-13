//! Universal SysEx parsing. These are the only messages specified in the
//! MIDI spec itself, rather than by a manufacturer.
//!
//! The main reference here was the _MIDI 1.0 Detailed Specification_.

use super::{
    ManufacturerId, StaticSysExGenerator, SysExGenerator, SysExGeneratorMenuTrait,
    MF_ID_UNIVERSAL_NON_REAL_TIME,
};
use crate::midi::format_bytes;
use crate::ui::{Menu, MenuItemResult};
use std::fmt::{Display, Formatter, Result as FmtResult};

pub type DeviceId = u8;
/// "All call" is the name in the MIDI 1.0 Detailed Specification, but it is
/// more intuitive to call this the "broadcast" ID. That's what Roland do.
pub const DV_ID_BROADCAST: ManufacturerId = 0x7F;

pub type SubId1 = u8;

// Non-real time message sub-ID#1 values. The real time messages use different
// meanings for this byte! TODO: add constants for those too.

// Unused (00h) deliberately skipped
pub const SI1_NRT_SAMPLE_DUMP_HEADER: SubId1 = 0x01;
pub const SI1_NRT_SAMPLE_DATA_PACKET: SubId1 = 0x02;
pub const SI1_NRT_SAMPLE_DUMP_REQUEST: SubId1 = 0x03;
pub const SI1_NRT_MIDI_TIME_CODE: SubId1 = 0x04;
pub const SI1_NRT_SAMPLE_DUMP_EXTENSIONS: SubId1 = 0x05;
pub const SI1_NRT_GENERAL_INFORMATION: SubId1 = 0x06;
pub const SI1_NRT_FILE_DUMP: SubId1 = 0x07;
pub const SI1_NRT_MIDI_TUNING_STANDARD: SubId1 = 0x08;
pub const SI1_NRT_GENERAL_MIDI: SubId1 = 0x09;
pub const SI1_NRT_END_OF_FILE: SubId1 = 0x7B;
pub const SI1_NRT_WAIT: SubId1 = 0x7C;
pub const SI1_NRT_CANCEL: SubId1 = 0x7D;
pub const SI1_NRT_NAK: SubId1 = 0x7E;
pub const SI1_NRT_ACK: SubId1 = 0x7F;

pub type SubId2 = u8;

// Sub-ID#2 values are namespaced under Sub-ID#1 ones.  These are the
// General MIDI ones.
pub const SI2_NRT_GM_GENERAL_MIDI_SYSTEM_ON: SubId2 = 0x01;
pub const SI2_NRT_GM_GENERAL_MIDI_SYSTEM_OFF: SubId2 = 0x02;

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
            real_time,
            device_id,
            sub_id1,
            sub_id2,
            data,
        } = self;

        if device_id == DV_ID_BROADCAST {
            write!(f, "Broadcast, ")?;
        } else {
            write!(f, "Device {:02X}h, ", device_id)?;
        }
        match (real_time, sub_id1) {
            (false, SI1_NRT_SAMPLE_DUMP_HEADER) => write!(f, "Sample Dump Header")?,
            (false, SI1_NRT_SAMPLE_DATA_PACKET) => write!(f, "Sample Data Packet")?,
            (false, SI1_NRT_SAMPLE_DUMP_REQUEST) => write!(f, "Sample Dump Request")?,
            (false, SI1_NRT_MIDI_TIME_CODE) => write!(f, "MIDI Time Code")?,
            (false, SI1_NRT_SAMPLE_DUMP_EXTENSIONS) => write!(f, "Sample Dump Extensions")?,
            (false, SI1_NRT_GENERAL_INFORMATION) => write!(f, "General Information")?,
            (false, SI1_NRT_FILE_DUMP) => write!(f, "File Dump")?,
            (false, SI1_NRT_MIDI_TUNING_STANDARD) => write!(f, "MIDI Tuning Standard")?,
            (false, SI1_NRT_GENERAL_MIDI) => write!(f, "General MIDI")?,
            (false, SI1_NRT_END_OF_FILE) => write!(f, "End Of File")?,
            (false, SI1_NRT_WAIT) => write!(f, "Wait")?,
            (false, SI1_NRT_CANCEL) => write!(f, "Cancel")?,
            (false, SI1_NRT_NAK) => write!(f, "NAK")?,
            (false, SI1_NRT_ACK) => write!(f, "ACK")?,
            (false, _) => write!(f, "Sub-ID#1 (unknown) {:02X}h", sub_id1)?,
            // We don't have constants for the real-time ones so we can't
            // meaningfully say they're unknown.
            (true, _) => write!(f, "Sub-ID#1 {:02X}h", sub_id1)?,
        }
        match (real_time, sub_id1, sub_id2) {
            (false, SI1_NRT_GENERAL_MIDI, SI2_NRT_GM_GENERAL_MIDI_SYSTEM_ON) => {
                write!(f, ", General MIDI System On")?
            }
            (false, SI1_NRT_GENERAL_MIDI, SI2_NRT_GM_GENERAL_MIDI_SYSTEM_OFF) => {
                write!(f, ", General MIDI System Off")?
            }
            _ => write!(f, ", Sub-ID#2 {:02X}h", sub_id2)?,
        }
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

pub(super) fn generate_nrt_sysex() -> Box<SysExGeneratorMenuTrait> {
    struct SysExGeneratorMenu;

    #[allow(clippy::type_complexity)]
    const SYSEX_GENERATORS: &[(&str, fn() -> Box<SysExGeneratorMenuTrait>)] =
        &[("General MIDI (@ Broadcast)", generate_general_midi_sysex)];

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

    Box::new(SysExGeneratorMenu)
}

fn generate_general_midi_sysex() -> Box<SysExGeneratorMenuTrait> {
    struct SysExGeneratorMenu;

    #[allow(clippy::type_complexity)]
    const SYSEX_GENERATORS: &[(&str, fn() -> Box<dyn SysExGenerator>)] =
        &[("General MIDI System On", || {
            Box::new(StaticSysExGenerator(&[
                0xF0,
                MF_ID_UNIVERSAL_NON_REAL_TIME,
                DV_ID_BROADCAST,
                SI1_NRT_GENERAL_MIDI,
                SI2_NRT_GM_GENERAL_MIDI_SYSTEM_ON,
                0xF7,
            ]))
        })];

    impl Menu<Box<dyn SysExGenerator>> for SysExGeneratorMenu {
        fn items_count(&self) -> usize {
            SYSEX_GENERATORS.len()
        }
        fn item_label(&self, item_idx: usize, write_to: &mut dyn std::fmt::Write) -> FmtResult {
            write!(write_to, "{}", SYSEX_GENERATORS[item_idx].0)
        }
        fn item_descend(&self, item_idx: usize) -> MenuItemResult<Box<dyn SysExGenerator>> {
            MenuItemResult::Command(SYSEX_GENERATORS[item_idx].1())
        }
    }

    Box::new(SysExGeneratorMenu)
}
