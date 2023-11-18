//! Various Parameter Address Maps.
//!
//! TODO: These should probably be stored as data files?

use super::{
    AddressBlockMap, ModelInfo, Parameter, ParameterAddressMap, ParameterValueDescription,
};

const fn param_unsigned(
    lsb: &'static [u8],
    size: u8,
    name: &'static str,
    range: std::ops::RangeInclusive<u8>,
) -> (&'static [u8], Parameter) {
    if size != 0x01 {
        panic!(); // only single-byte for now
    }
    (
        lsb,
        Parameter {
            size,
            name,
            range,
            description: ParameterValueDescription::Numeric {
                zero_offset: 0,
                unit_in_range: None,
            },
        },
    )
}
const fn param_signed(
    lsb: &'static [u8],
    size: u8,
    name: &'static str,
    range: std::ops::RangeInclusive<u8>,
    zero_offset: u8,
) -> (&'static [u8], Parameter) {
    if size != 0x01 {
        panic!(); // only single-byte for now
    }
    (
        lsb,
        Parameter {
            size,
            name,
            range,
            description: ParameterValueDescription::Numeric {
                zero_offset,
                unit_in_range: None,
            },
        },
    )
}
const fn param_range(
    lsb: &'static [u8],
    size: u8,
    name: &'static str,
    range_midi: std::ops::RangeInclusive<u8>,
    zero_midi: u8,
    range_unit: std::ops::RangeInclusive<f32>,
    unit: &'static str,
) -> (&'static [u8], Parameter) {
    if size != 0x01 {
        panic!(); // only single-byte for now
    }
    (
        lsb,
        Parameter {
            size,
            name,
            range: range_midi,
            description: ParameterValueDescription::Numeric {
                zero_offset: zero_midi,
                unit_in_range: Some((range_unit, unit)),
            },
        },
    )
}
const fn param_enum(
    lsb: &'static [u8],
    size: u8,
    name: &'static str,
    range: std::ops::RangeInclusive<u8>,
    values: &'static [(&'static [u8], &'static str)],
) -> (&'static [u8], Parameter) {
    if size != 0x01 {
        panic!(); // only single-byte for now
    }

    let mut value_min = None::<u8>;
    let mut value_max = None::<u8>;
    let mut i = 0;
    while i < values.len() {
        let value = values[i].0;
        let &[value] = value else {
            panic!();
        };

        match value_min {
            None => value_min = Some(value),
            Some(min) if value < min => value_min = Some(value),
            _ => (),
        }
        match value_max {
            None => value_max = Some(value),
            Some(max) if value > max => value_max = Some(value),
            _ => (),
        }

        i += 1;
    }

    // We could just generate the range from the values, but the references
    // specify both the range and the values, so this is a useful check that the
    // data is correct. Also, maybe we'll need to support enums that only have
    // partial coverage of the range, at some point.
    match (value_min, value_max) {
        (Some(value_min), Some(value_max))
            if value_min == *range.start() && value_max == *range.end() => {}
        _ => panic!(),
    }

    (
        lsb,
        Parameter {
            size,
            name,
            range,
            description: ParameterValueDescription::Enum(values),
        },
    )
}
// Only use this when it exactly matches the manual. Other single-byte two-value
// enums should use param_enum.
const fn param_bool(lsb: &'static [u8], name: &'static str) -> (&'static [u8], Parameter) {
    param_enum(
        lsb,
        0x01,
        name,
        0x00..=0x01,
        &[(&[0x00], "OFF"), (&[0x01], "ON")],
    )
}
const fn param_other(
    lsb: &'static [u8],
    size: u8,
    name: &'static str,
    range: std::ops::RangeInclusive<u8>,
) -> (&'static [u8], Parameter) {
    (
        lsb,
        Parameter {
            size,
            name,
            range,
            description: ParameterValueDescription::Other,
        },
    )
}

mod gs;
mod sc_55;
mod sc_7;

pub const MODELS: &[&ModelInfo] = &[&gs::GS, &sc_55::SC_55, &sc_7::SC_7];
