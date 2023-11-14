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

pub const MODELS: &[&ModelInfo] = &[&SC_7, &SC_55, &GS];

/// Roland SC-7, according to the SC-7 owner's manual. This device also uses
/// the GS model ID for some things.
const SC_7: ModelInfo = ModelInfo {
    model_id: &[0x56],
    name: "Roland SC-7",
    default_device_id: 0x10, // non-configurable
    address_size: 3,
    address_block_map: SC_7_ABM,
};

/// Roland SC-55/SC-155, according to the SC-55 and SC-55mkII owner's manuals.
/// This device also uses the GS model ID for some things.
const SC_55: ModelInfo = ModelInfo {
    model_id: &[0x45],
    name: "Roland SC-55/SC-155",
    default_device_id: 0x10,
    address_size: 3,
    address_block_map: &[], // TODO
};

/// Roland GS, according to the SC-55 and SC-55mkII owner's manuals.
const GS: ModelInfo = ModelInfo {
    model_id: &[0x42],
    name: "Roland GS",
    default_device_id: 0x10, // SC-55 and SC-7 respond to this, at least
    address_size: 3,
    address_block_map: &[], // TODO: add at least the SC-7 stuff
};

const SC_7_ABM: AddressBlockMap = &[
    (
        &[0x00, 0x00],
        "System parameters, Effect Control",
        SC_7_PAM_SYSTEM,
    ),
    (
        &[0x01, 0x00],
        "Patch parameters, Patch block 0",
        SC_7_PAM_PATCH,
    ),
    (
        &[0x01, 0x01],
        "Patch parameters, Patch block 1",
        SC_7_PAM_PATCH,
    ),
    (
        &[0x01, 0x02],
        "Patch parameters, Patch block 2",
        SC_7_PAM_PATCH,
    ),
    (
        &[0x01, 0x03],
        "Patch parameters, Patch block 3",
        SC_7_PAM_PATCH,
    ),
    (
        &[0x01, 0x04],
        "Patch parameters, Patch block 4",
        SC_7_PAM_PATCH,
    ),
    (
        &[0x01, 0x05],
        "Patch parameters, Patch block 5",
        SC_7_PAM_PATCH,
    ),
    (
        &[0x01, 0x06],
        "Patch parameters, Patch block 6",
        SC_7_PAM_PATCH,
    ),
    (
        &[0x01, 0x07],
        "Patch parameters, Patch block 7",
        SC_7_PAM_PATCH,
    ),
    (
        &[0x01, 0x08],
        "Patch parameters, Patch block 8",
        SC_7_PAM_PATCH,
    ),
    (
        &[0x01, 0x09],
        "Patch parameters, Patch block 9",
        SC_7_PAM_PATCH,
    ),
    (
        &[0x01, 0x0A],
        "Patch parameters, Patch block A",
        SC_7_PAM_PATCH,
    ),
    (
        &[0x01, 0x0B],
        "Patch parameters, Patch block B",
        SC_7_PAM_PATCH,
    ),
    (
        &[0x01, 0x0C],
        "Patch parameters, Patch block C",
        SC_7_PAM_PATCH,
    ),
    (
        &[0x01, 0x0D],
        "Patch parameters, Patch block D",
        SC_7_PAM_PATCH,
    ),
    (
        &[0x01, 0x0E],
        "Patch parameters, Patch block E",
        SC_7_PAM_PATCH,
    ),
    (
        &[0x01, 0x0F],
        "Patch parameters, Patch block F",
        SC_7_PAM_PATCH,
    ),
];

const SC_7_PAM_SYSTEM: ParameterAddressMap = &[
    param_enum(
        &[0x00],
        0x01,
        "REVERB CHARACTER",
        0x00..=0x07,
        &[
            (&[0x00], "Room 1"),
            (&[0x01], "Room 2"),
            (&[0x02], "Room 3"),
            (&[0x03], "Hall 1"),
            (&[0x04], "Hall 2"),
            (&[0x05], "Plate"),
            (&[0x06], "Delay"),
            (&[0x07], "Panning Delay"),
        ],
    ),
    param_simple(&[0x01], 0x01, "REVERB LEVEL", None),
    param_simple(&[0x02], 0x01, "REVERB (DELAY) TIME", None),
    param_simple(&[0x03], 0x01, "DELAY TIME", None),
    param_simple(&[0x04], 0x01, "DELAY FEEDBACK", None),
    param_simple(&[0x05], 0x01, "CHORUS LEVEL", None),
    param_simple(&[0x06], 0x01, "CHORUS FEEDBACK", None),
    param_simple(&[0x07], 0x01, "CHORUS DELAY", None),
    param_simple(&[0x08], 0x01, "CHORUS RATE", None),
    param_simple(&[0x09], 0x01, "CHORUS DEPTH", None),
];

const SC_7_PAM_PATCH: ParameterAddressMap = &[
    param_enum(
        &[0x00],
        0x01,
        "RX. CHANNEL",
        0x00..=0x10,
        &[
            (&[0x00], "Channel 1"),
            (&[0x01], "Channel 2"),
            (&[0x02], "Channel 3"),
            (&[0x03], "Channel 4"),
            (&[0x04], "Channel 5"),
            (&[0x05], "Channel 6"),
            (&[0x06], "Channel 7"),
            (&[0x07], "Channel 8"),
            (&[0x08], "Channel 9"),
            (&[0x09], "Channel 10"),
            (&[0x0A], "Channel 11"),
            (&[0x0B], "Channel 12"),
            (&[0x0C], "Channel 13"),
            (&[0x0D], "Channel 14"),
            (&[0x0E], "Channel 15"),
            (&[0x0F], "Channel 16"),
            (&[0x10], "OFF"),
        ],
    ),
    param_enum(
        &[0x01],
        0x01,
        "RX. NRPN",
        0x00..=0x01,
        &[(&[0x00], "OFF"), (&[0x01], "ON")],
    ),
    param_simple(&[0x02], 0x01, "MOD LFO RATE CONTROL", None),
    param_simple(&[0x03], 0x01, "MOD LFO PITCH DEPTH", None),
    param_simple(&[0x04], 0x01, "CAF TVF CUT OFF CONTROL", None),
    param_simple(&[0x05], 0x01, "CAF AMPLITUDE CONTROL", None),
    param_simple(&[0x06], 0x01, "CAF LFO RATE CONTROL", None),
    param_simple(&[0x07], 0x01, "CAF LFO PITCH DEPTH", None),
];
