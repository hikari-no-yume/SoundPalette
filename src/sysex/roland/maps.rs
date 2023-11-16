//! Various Parameter Address Maps.
//!
//! TODO: These should probably be stored as data files?

use super::{
    AddressBlockMap, ModelInfo, Parameter, ParameterAddressMap, ParameterValueDescription,
};

const fn param_simple(
    lsb: &'static [u8],
    size: u8,
    name: &'static str,
    range: Option<std::ops::RangeInclusive<u8>>,
) -> (&'static [u8], Parameter) {
    (
        lsb,
        Parameter {
            size,
            name,
            range,
            description: ParameterValueDescription::Simple,
        },
    )
}
const fn param_range(
    lsb: &'static [u8],
    size: u8,
    name: &'static str,
    range_midi: std::ops::RangeInclusive<u8>,
    zero_midi: Option<u8>,
    range_unit: std::ops::RangeInclusive<f32>,
    unit: &'static str,
) -> (&'static [u8], Parameter) {
    (
        lsb,
        Parameter {
            size,
            name,
            range: Some(range_midi),
            description: ParameterValueDescription::UnitInRange(range_unit, unit, zero_midi),
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
            range: Some(range),
            description: ParameterValueDescription::Enum(values),
        },
    )
}

/*macro_rules! param_simple {
    ($lsb:expr, $size:expr, $name:expr) => {
        (
            $lsb,
            Parameter {
                size: $size,
                name: $name,
                range: None,
                description: ParameterValueDescription,
            },
        )
    };
}

macro_rules! param_range {
    ($lsb:expr, $size:expr, $name:expr, $range:expr) => {
        (
            $lsb,
            Parameter {
                size: $size,
                name: $name,
                range: $range,
                description: ParameterValueDescription::Simple,
            },
        )
    };
}

macro_rules! param_enum {
    ($lsb:expr, $size:expr, $name:expr, $range:expr) => {
        (
            $lsb,
            Parameter {
                size: $size,
                name: $name,
                range: $range,
                description: ParameterValueDescription::Simple,
            },
        )
    };
}*/

mod gs;
mod sc_55;
mod sc_7;

pub const MODELS: &[&ModelInfo] = &[&gs::GS, &sc_55::SC_55, &sc_7::SC_7];
