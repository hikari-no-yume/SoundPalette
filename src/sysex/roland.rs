/*
 * Part of SoundPalette by hikari_no_yume.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! Roland SysEx parsing.
//!
//! My first reference for this is the Roland SC-7 owner's manual, and I have
//! also looked at the SC-55 and SC-55mkII owner's manuals. The same basic
//! protocol is used by all of these devices AFAICT, the differences are just in
//! the device IDs and "parameter maps". Presumably the rest of the Sound Canvas
//! series use this too. I don't know about other Roland devices.

use super::{
    ManufacturerId, MaybeParsed, ParsedSysEx, ParsedSysExBody, SysExGenerator,
    SysExGeneratorMenuTrait,
};
use crate::midi::format_bytes;
use crate::ui::{Menu, MenuItemResult};
use std::fmt::{Display, Formatter, Result as FmtResult};

pub const MF_ID_ROLAND: ManufacturerId = 0x41;

pub type DeviceId = u8;

/// Variable-length quantity (see [consume_variable_length_id]).
pub type ModelId<'a> = &'a [u8];

/// Variable-length quantity (see [consume_variable_length_id]).
pub type CommandId<'a> = &'a [u8];

/// "Data set 1" aka "DT1".
pub const CM_ID_DT1: CommandId<'static> = &[0x12];

// TODO: support "Request data 1" aka "RQ1".

#[derive(Debug)]
pub enum ParsedRolandSysExBody<'a> {
    /// Roland SC-7 manual says "Roland's MIDI implementation uses the following
    /// data format for all Exclusive messages" and refers to it as "Type IV".
    /// You can see similar text in many other Roland product manuals, including
    /// the SC-55 for example. I don't know where this numbering comes from.
    ///
    /// The `device_id`, `model_id` and `command_id` are raw parsing results.
    /// The `model_name` is an interpretation that is the result of a lookup.
    /// `command` is a hybrid of course.
    TypeIV {
        device_id: DeviceId,
        model_id: ModelId<'a>,
        model_name: Option<&'static str>,
        command_id: CommandId<'a>,
        command: MaybeParsed<'a, ParsedRolandSysExCommand<'a>>,
    },
}
impl Display for ParsedRolandSysExBody<'_> {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            &ParsedRolandSysExBody::TypeIV {
                device_id,
                model_id,
                model_name,
                command_id,
                ref command,
            } => {
                write!(f, "Device {:02X}h, ", device_id)?;
                match model_name {
                    Some(model_name) => write!(f, "{}", model_name)?,
                    _ => write!(f, "Model {}", format_bytes(model_id))?,
                }
                if let MaybeParsed::Unknown(_) = command {
                    write!(f, ", Command {}", format_bytes(command_id))?
                }
                write!(f, ": {}", command)?;
            }
        }
        Ok(())
    }
}

/// The Model ID and Command ID fields in Roland Exclusive Messages can have a
/// 00h prefix to extend the length. It's not a very efficient variable-length
/// integer encoding because it's not positional, the prefix only gives you
/// another 126 values total in your encoding space for each use.
fn consume_variable_length_id(data: &[u8]) -> Result<(&[u8], &[u8]), ()> {
    let mut id_end = 1;
    loop {
        if id_end > data.len() {
            return Err(());
        } else if data[id_end - 1] == 0x00 {
            id_end += 1;
            continue;
        } else {
            return Ok((&data[..id_end], &data[id_end..]));
        }
    }
}

#[allow(clippy::result_unit_err)] // not much explanation can be given really
pub fn parse_sysex_body(body: &[u8]) -> Result<ParsedRolandSysExBody, ()> {
    let (&device_id, body) = body.split_first().ok_or(())?;
    let (model_id, body) = consume_variable_length_id(body)?;
    let (command_id, body) = consume_variable_length_id(body)?;

    let model_info = MODELS.iter().find(|model| model.model_id == model_id);

    // Command parsing needs model info in order to know e.g. how large an
    // address is.
    let command = match model_info
        .ok_or(())
        .and_then(|model_info| parse_sysex_command(model_info, command_id, body))
    {
        Ok(parsed) => MaybeParsed::Parsed(parsed),
        Err(()) => MaybeParsed::Unknown(body),
    };

    Ok(ParsedRolandSysExBody::TypeIV {
        device_id,
        model_id,
        model_name: model_info.map(|model| model.name),
        command_id,
        command,
    })
}

impl SysExGenerator for ParsedRolandSysExBody<'_> {
    fn generate(&self, out: &mut Vec<u8>) {
        let &ParsedRolandSysExBody::TypeIV {
            device_id,
            model_id,
            command_id,
            ref command,
            // meaningless
            model_name: _,
        } = self;
        out.push(device_id);
        out.extend_from_slice(model_id);
        out.extend_from_slice(command_id);
        command.generate(out);
    }
}

#[derive(Debug)]
pub enum ParsedRolandSysExCommand<'a> {
    /// "Data set 1" aka "DT1". The `address` and `data` are the raw parsing
    /// results, whereas the other fields are interpretations that are only
    /// available and meaningful if a lookup succeeds, and can't be assumed to
    /// always be correct.
    ///
    /// Errors like incorrect checksum or invalid size/data are tolerated
    /// because this is helpful for troubleshooting when writing SysExes, and
    /// because this parser is not omniscient and might e.g. not know about how
    /// a parameter was changed in a newer model.
    DT1 {
        address: &'a [u8],
        data: &'a [u8],
        /// Was the checksum correct?
        valid_checksum: bool,
        /// Name of the parameter block the address seems to be for, if it could
        /// be found, and how many bytes of the address (starting from 0) it
        /// takes up.
        block_name_and_prefix_size: Option<(&'static str, u8)>,
        /// Information about the parameter the address seems to be for, if it
        /// could be found.
        param_info: Option<&'static Parameter>,
        /// Whether the size of the data matches the parameter info that was
        /// looked up.
        invalid_size: bool,
    },
}
impl ParsedRolandSysExCommand<'_> {
    /// Validate the data field only. Returns [true] if enough information is
    /// available to validate the data, and it indicates an error; a return
    /// value of [false] does not mean the data can't be invalid, and a return
    /// value of [true] does not mean the data is meaningless (e.g. SoundPalette
    /// might not know of changes to a parameter in a newer model).
    fn data_is_out_of_range(&self) -> bool {
        match self {
            &ParsedRolandSysExCommand::DT1 {
                address: _,
                data,
                valid_checksum: _,
                block_name_and_prefix_size: _,
                param_info: Some(Parameter { range, .. }),
                invalid_size: false,
            } => data.iter().any(|&data_byte| !range.contains(&data_byte)),
            _ => false,
        }
    }
}
impl Display for ParsedRolandSysExCommand<'_> {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            &ParsedRolandSysExCommand::DT1 {
                address,
                data,
                valid_checksum,
                block_name_and_prefix_size,
                param_info,
                invalid_size,
            } => {
                write!(f, "Data set 1: ")?;

                if let Some((block_name, prefix_size)) = block_name_and_prefix_size {
                    write!(f, "{} § ", block_name)?;
                    if let Some(param_info) = param_info {
                        write!(
                            f,
                            "{}{}",
                            param_info.name,
                            if invalid_size { " (WRONG SIZE)" } else { "" }
                        )?;
                    } else {
                        write!(
                            f,
                            "(unknown) {}",
                            format_bytes(&address[prefix_size as usize..])
                        )?;
                    }
                } else {
                    assert!(param_info.is_none());
                    assert!(!invalid_size);
                    write!(f, "(unknown) {}", format_bytes(address))?;
                }

                write!(f, " => {}", format_bytes(data))?;

                if !invalid_size {
                    if let Some(param_info) = param_info {
                        param_info.describe(data, f, false)?;
                    }
                }

                if self.data_is_out_of_range() {
                    write!(f, " (out of range")?;
                }
                if !valid_checksum {
                    write!(f, " (WRONG CHECKSUM)")?;
                }
            }
        }
        Ok(())
    }
}

fn compute_checksum(data: &[u8]) -> u8 {
    let mut sum: u8 = 0;
    for &byte in data {
        sum = (sum + byte) & 0x7F;
    }
    sum
}
pub fn validate_checksum(data_including_checksum: &[u8]) -> bool {
    compute_checksum(data_including_checksum) == 0
}
pub fn generate_checksum(data_without_checksum: &[u8]) -> u8 {
    (0x80 - compute_checksum(data_without_checksum)) & 0x7F
}

#[allow(clippy::result_unit_err)] // not much explanation can be given really
pub fn parse_sysex_command<'a>(
    model_info: &ModelInfo,
    command_id: CommandId,
    body: &'a [u8],
) -> Result<ParsedRolandSysExCommand<'a>, ()> {
    match command_id {
        CM_ID_DT1 => {
            // The body must be large enough have an address and a checksum
            // byte. Not sure if data values can be zero bytes long, but why
            // not?

            let address_end = model_info.address_size as usize;
            if address_end > body.len() {
                return Err(());
            }
            let checksum_begin = body.len() - 1;
            if checksum_begin < address_end {
                return Err(());
            }
            let address = &body[..address_end];
            let data = &body[address_end..checksum_begin];

            let valid_checksum = validate_checksum(body);
            let (block_name_and_prefix_size, param_info) = look_up_parameter(model_info, address);
            let invalid_size = param_info.map_or(false, |param| param.size as usize != data.len());

            Ok(ParsedRolandSysExCommand::DT1 {
                address,
                data,
                valid_checksum,
                block_name_and_prefix_size,
                param_info,
                invalid_size,
            })
        }
        _ => Err(()),
    }
}

impl SysExGenerator for ParsedRolandSysExCommand<'_> {
    fn generate(&self, out: &mut Vec<u8>) {
        let &ParsedRolandSysExCommand::DT1 {
            address,
            data,
            // meaningless stuff
            valid_checksum: _,
            block_name_and_prefix_size: _,
            param_info: _,
            invalid_size: _,
        } = self;

        let command_start = out.len();
        out.extend_from_slice(address);
        out.extend_from_slice(data);
        out.push(generate_checksum(&out[command_start..]));
    }
}

/// Uses [MODELS] to look up the name of the address block, the size of its
/// address prefix, and the details of the parameter using an address, if
/// possible.
pub fn look_up_parameter(
    model_info: &ModelInfo,
    address: &[u8],
) -> (Option<(&'static str, u8)>, Option<&'static Parameter>) {
    let Some((lsb, block_name, pam)) =
        model_info
            .address_block_map
            .iter()
            .find_map(|&(msb, block_name, pam)| {
                address.strip_prefix(msb).map(|lsb| (lsb, block_name, pam))
            })
    else {
        return (None, None);
    };

    (
        Some((block_name, (address.len() - lsb.len()).try_into().unwrap())),
        pam.iter()
            .find(|&&(lsb2, _)| lsb == lsb2)
            .map(|(_, param)| param),
    )
}

/// Model-specific information.
///
/// `address_size` is the number of bytes used by an address for a DT1 command.
/// This is constant for a particular model, but varies between models.
///
/// `default_device_id` is the default, or sometimes only, device ID for this
/// model. I've only seen `10h` but it seems reasonable to parameterise it.
#[derive(Debug)]
pub struct ModelInfo {
    pub model_id: ModelId<'static>,
    pub name: &'static str,
    pub default_device_id: DeviceId,
    pub address_size: u8,
    pub address_block_map: AddressBlockMap,
}

/// "Address Block Map" in the style of the Roland SC-7 owner's manual.
/// Describes the high-level layout of the parameter map via address prefixes
/// (most significant bytes). Each block has a human-readable name.
pub type AddressBlockMap = &'static [(&'static [u8], &'static str, ParameterAddressMap)];

/// "Parameter Block Map" in the style of the Roland SC-7 owner's manual.
/// Describes the low-level layout of the parameter map via address suffixes
/// (least significant bytes). See also [AddressBlockMap].
pub type ParameterAddressMap = &'static [(&'static [u8], Parameter)];

/// The rows from a "Parameter Address Map" (see [ParameterAddressMap]).
#[derive(Debug)]
pub struct Parameter {
    /// "Size": Number of data bytes expected for this parameter
    pub size: u8,
    /// "Name": Human-readable name for this parameter
    pub name: &'static str,
    /// Range of valid values for the data bytes of this parameter, from the
    /// "Data" column. This is a [std::ops::RangeInclusive] because it's the
    /// style used in Roland documentation and it's compact.
    pub range: std::ops::RangeInclusive<u8>,
    /// "Description": a meaning for the values of this parameter.
    /// Please ensure this matches the range.
    pub description: ParameterValueDescription,
    // TODO: Default Value?
}

/// Meaning for the values of a parameter, trying to match the "Description" of
/// a "Parameter Address Map".
#[derive(Debug)]
pub enum ParameterValueDescription {
    /// Simple numeric value. Often, the meaning of this parameter's value isn't
    /// described beyond giving a name to the parameter. Display this as a
    /// decimal integer, like the manuals. Currently this is only used for
    /// single-byte parameters.
    ///
    /// `zero_offset` specifies the offset used for biased integer
    /// representation of negative values. If this is zero, the value is always
    /// positive.
    ///
    /// Some parameters' values are mapped to a range in some particular unit.
    /// The range can be bigger, smaller, or the same size, so the mapping is
    /// usually unspecified and approximate. In these cases, `unit_in_range`
    /// gives the name of the unit and a range in that unit to map to.
    Numeric {
        zero_offset: u8,
        unit_in_range: Option<(std::ops::RangeInclusive<f32>, &'static str)>,
    },
    /// There is an enumerated list of values for this parameter.
    Enum(&'static [(&'static [u8], &'static str)]),
    /// Something else that isn't handled yet.
    Other,
}

impl Parameter {
    /// Write a human-readable description of the data `data`, if interpreted as
    /// a value for this parameter, to `write_to`. If the result is not empty,
    /// it always begins with a space, usually followed by an equals sign and a
    /// decimal value. `em_dash` is [true] if an em dash should be used to
    /// separate definitions from raw values, otherwise square brackets are
    /// used. The data must be the right size.
    pub fn describe(
        &self,
        data: &[u8],
        write_to: &mut (impl std::fmt::Write + ?Sized),
        em_dash: bool,
    ) -> FmtResult {
        assert_eq!(data.len(), self.size as usize);

        let zero_offset = match self.description {
            ParameterValueDescription::Numeric { zero_offset, .. } => zero_offset,
            ParameterValueDescription::Enum(_) => 0,
            ParameterValueDescription::Other => return Ok(()),
        };

        let differing_signs_at_range_ends =
            zero_offset != *self.range.start() && zero_offset != *self.range.end();

        if let &[single_byte_value] = data {
            if differing_signs_at_range_ends {
                write!(
                    write_to,
                    " = {:+}",
                    (single_byte_value as i16) - zero_offset as i16
                )?;
            } else {
                write!(
                    write_to,
                    " = {}",
                    (single_byte_value as i16) - zero_offset as i16
                )?;
            }
        }

        match self.description {
            ParameterValueDescription::Enum(values) => {
                if let Some(&(_, name)) = values.iter().find(|&&(data2, _)| data2 == data) {
                    if em_dash {
                        write!(write_to, " — {}", name)?;
                    } else {
                        write!(write_to, " [{}]", name)?;
                    }
                }
            }
            ParameterValueDescription::Numeric {
                zero_offset: midi_zero,
                unit_in_range: Some((ref unit_range, unit)),
            } => {
                let &[midi_value] = data else {
                    todo!();
                };
                let midi_value = midi_value as f32;

                let midi_range = &self.range;
                assert!(midi_range.start() < midi_range.end());
                let midi_min = *midi_range.start() as f32;
                let midi_max = *midi_range.end() as f32;
                let midi_range = midi_max - midi_min;

                // TODO: Support range flips eventually?
                assert!(unit_range.start() < unit_range.end());
                let unit_min: f32 = *unit_range.start();
                let unit_max: f32 = *unit_range.end();
                let unit_range = unit_max - unit_min;

                // The exact way the MIDI data byte maps to the actual unit is
                // not specified, hence the “approximately equal to” sign and
                // imprecise figures. I don't and can't know if this rounding is
                // correct. The most important property is that it rounds the
                // known zero value to zero, to avoid confusion for the very
                // common case where the range is symmetrical in the destination
                // unit but not in the MIDI byte values, e.g. -20Hz to +20Hz
                // versus 00h to 7Fh with with zero at 40h.
                let unit_value = (midi_value - midi_zero as f32) * (unit_range / midi_range);

                // Occasionally the mapping is actually exact (e.g. key shift)
                if unit_range == midi_range {
                    write!(write_to, " [= ")?;
                } else {
                    write!(write_to, " [≈ ")?;
                }

                // In order to not imply more precision than we actually have,
                // only add decimal places if they're necessary to convey
                // differences between steps.
                let precision = (midi_range.log10() - unit_range.log10()).ceil().max(0.0) as usize;

                let differing_signs_at_range_ends = unit_min < 0.0 && unit_max > 0.0;

                if unit_value == 0.0 {
                    write!(write_to, "0")?;
                } else if differing_signs_at_range_ends {
                    write!(write_to, "{:+.*}", precision, unit_value)?;
                } else {
                    write!(write_to, "{:.*}", precision, unit_value)?;
                }

                write!(write_to, " {}]", unit)?;
            }
            _ => (),
        }

        Ok(())
    }
}

// All the maps are in their own module to keep this one small.
mod maps;
pub use maps::MODELS;

/// Provides a menu for generating a SysEx.
pub fn generate_sysex() -> Box<SysExGeneratorMenuTrait> {
    // These are nested like Matryoshki because the amount of state needed is
    // strictly increasing with each step.
    struct ModelsMenu;
    #[derive(Clone, Debug)]
    struct AddressBlockMenu {
        model_info: &'static ModelInfo,
    }
    #[derive(Clone, Debug)]
    struct ParameterAddressMenu {
        up: AddressBlockMenu,
        address_prefix: &'static [u8],
        parameter_address_map: ParameterAddressMap,
    }
    #[derive(Clone, Debug)]
    struct ParameterValueMenu {
        up: ParameterAddressMenu,
        address_suffix: &'static [u8],
        param: &'static Parameter,
    }
    #[derive(Debug)]
    struct DT1Generator {
        up: ParameterValueMenu,
        value: u8,
    }

    impl Menu<Box<dyn SysExGenerator>> for ModelsMenu {
        fn items_count(&self) -> usize {
            MODELS.len()
        }
        fn item_label(&self, item_idx: usize, write_to: &mut dyn std::fmt::Write) -> FmtResult {
            let ModelInfo {
                model_id,
                name,
                default_device_id,
                ..
            } = MODELS[item_idx];
            write!(
                write_to,
                "{} — {} (@ Device {:02X}h)",
                format_bytes(model_id),
                name,
                default_device_id
            )
        }
        fn item_disabled(&self, item_idx: usize) -> bool {
            MODELS[item_idx].address_block_map.is_empty()
        }
        fn item_descend(&self, item_idx: usize) -> MenuItemResult<Box<dyn SysExGenerator>> {
            MenuItemResult::Submenu(Box::new(AddressBlockMenu {
                model_info: MODELS[item_idx],
            }))
        }
    }

    impl Menu<Box<dyn SysExGenerator>> for AddressBlockMenu {
        fn items_count(&self) -> usize {
            self.model_info.address_block_map.len()
        }
        fn item_label(&self, item_idx: usize, write_to: &mut dyn std::fmt::Write) -> FmtResult {
            let (address_prefix, name, _) = self.model_info.address_block_map[item_idx];
            write!(write_to, "{} — {}", format_bytes(address_prefix), name)
        }
        fn item_disabled(&self, item_idx: usize) -> bool {
            let (_, _, parameter_address_map) = self.model_info.address_block_map[item_idx];
            parameter_address_map.is_empty()
        }
        fn item_descend(&self, item_idx: usize) -> MenuItemResult<Box<dyn SysExGenerator>> {
            let (address_prefix, _, parameter_address_map) =
                self.model_info.address_block_map[item_idx];
            MenuItemResult::Submenu(Box::new(ParameterAddressMenu {
                up: self.clone(),
                address_prefix,
                parameter_address_map,
            }))
        }
    }

    impl Menu<Box<dyn SysExGenerator>> for ParameterAddressMenu {
        fn items_count(&self) -> usize {
            self.parameter_address_map.len()
        }
        fn item_label(&self, item_idx: usize, write_to: &mut dyn std::fmt::Write) -> FmtResult {
            let (address_suffix, ref param) = self.parameter_address_map[item_idx];
            write!(
                write_to,
                "{} — {}",
                format_bytes(address_suffix),
                param.name
            )
        }
        fn item_disabled(&self, item_idx: usize) -> bool {
            let (_, ref param) = self.parameter_address_map[item_idx];
            param.size != 1 || matches!(param.description, ParameterValueDescription::Other)
        }
        fn item_descend(&self, item_idx: usize) -> MenuItemResult<Box<dyn SysExGenerator>> {
            let (address_suffix, ref param) = self.parameter_address_map[item_idx];
            // TODO: support parameters that aren't a single byte long.
            assert_eq!(param.size, 1);
            MenuItemResult::Submenu(Box::new(ParameterValueMenu {
                up: self.clone(),
                address_suffix,
                param,
            }))
        }
    }

    impl ParameterValueMenu {
        fn values_range(&self) -> std::ops::Range<usize> {
            // Change from inclusive to exclusive end bound
            (*self.param.range.start() as usize)..(*self.param.range.end() as usize + 1)
        }
        fn item_value(&self, item_idx: usize) -> u8 {
            let value = self.values_range().start + item_idx;
            assert!(self.values_range().contains(&value));
            // Currently, values can only be single MIDI data bytes (7-bit)
            assert!(value < (1 << 7));
            u8::try_from(value).unwrap()
        }
    }
    impl Menu<Box<dyn SysExGenerator>> for ParameterValueMenu {
        fn items_count(&self) -> usize {
            self.values_range().end - self.values_range().start
        }
        fn item_label(&self, item_idx: usize, write_to: &mut dyn std::fmt::Write) -> FmtResult {
            let data = &[self.item_value(item_idx)];
            write!(write_to, "{}", format_bytes(data))?;
            self.param.describe(data, write_to, true)
        }
        fn item_descend(&self, item_idx: usize) -> MenuItemResult<Box<dyn SysExGenerator>> {
            MenuItemResult::Command(Box::new(DT1Generator {
                up: self.clone(),
                value: self.item_value(item_idx),
            }))
        }
    }

    impl SysExGenerator for DT1Generator {
        fn generate(&self, out: &mut Vec<u8>) {
            let mut address =
                Vec::with_capacity(self.up.up.address_prefix.len() + self.up.address_suffix.len());
            address.extend_from_slice(self.up.up.address_prefix);
            address.extend_from_slice(self.up.address_suffix);
            ParsedSysEx {
                manufacturer_id: MF_ID_ROLAND,
                content: MaybeParsed::Parsed(ParsedSysExBody::Roland(
                    ParsedRolandSysExBody::TypeIV {
                        device_id: self.up.up.up.model_info.default_device_id,
                        model_id: self.up.up.up.model_info.model_id,
                        model_name: None, // meaningless,
                        command_id: CM_ID_DT1,
                        command: MaybeParsed::Parsed(ParsedRolandSysExCommand::DT1 {
                            address: &address,
                            data: &[self.value],
                            param_info: Some(self.up.param),
                            // meaningless stuff
                            valid_checksum: false,
                            block_name_and_prefix_size: None,
                            invalid_size: false,
                        }),
                    },
                )),
            }
            .generate(out)
        }
    }

    Box::new(ModelsMenu)
}
