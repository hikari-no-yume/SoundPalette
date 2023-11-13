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
    /// results, whereas the other fields are interpretation.
    DT1 {
        address: &'a [u8],
        data: &'a [u8],
        /// Was the checksum correct? Wrong checksums are tolerated because this
        /// is more helpful in MIDI debugging than displaying no info.
        valid_checksum: bool,
        /// Name of the parameter block the address seems to be for, if it could
        /// be found, and how many bytes of the address (starting from 0) it
        /// takes up.
        block_name_and_prefix_size: Option<(&'static str, u8)>,
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
    // TODO: handle Data, Description, Default Value etc in some reasonable way
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

    impl Menu<Box<dyn SysExGenerator>> for ParameterValueMenu {
        fn items_count(&self) -> usize {
            // Values are just valid MIDI data byte values (7-bit), currently.
            1 << 7
        }
        fn item_label(&self, item_idx: usize, write_to: &mut dyn std::fmt::Write) -> FmtResult {
            write!(write_to, "{:02X}h = {}", item_idx, item_idx)
        }
        fn item_descend(&self, item_idx: usize) -> MenuItemResult<Box<dyn SysExGenerator>> {
            assert!(item_idx < (1 << 7));
            let value = item_idx.try_into().unwrap();

            MenuItemResult::Command(Box::new(DT1Generator {
                up: self.clone(),
                value,
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
