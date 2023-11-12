//! Various Parameter Address Maps.
//!
//! TODO: These should probably be stored as data files?

use super::{AddressBlockMap, ModelInfo, Parameter, ParameterAddressMap};

macro_rules! param {
    ($lsb:expr, $size:expr, $name:expr) => {
        (
            $lsb,
            Parameter {
                size: $size,
                name: $name,
            },
        )
    };
}

pub const MODELS: &[&ModelInfo] = &[&SC_7, &SC_55, &GS];

/// Roland SC-7, according to the SC-7 owner's manual. This device also uses
/// the GS model ID for some things.
const SC_7: ModelInfo = ModelInfo {
    model_id: &[0x56],
    name: "Roland SC-7",
    address_size: 3,
    address_block_map: SC_7_ABM,
};

/// Roland SC-55/SC-155, according to the SC-55 and SC-55mkII owner's manuals.
/// This device also uses the GS model ID for some things.
const SC_55: ModelInfo = ModelInfo {
    model_id: &[0x45],
    name: "Roland SC-55/SC-155",
    address_size: 3,
    address_block_map: &[], // TODO
};

/// Roland GS, according to the SC-55 and SC-55mkII owner's manuals.
const GS: ModelInfo = ModelInfo {
    model_id: &[0x42],
    name: "Roland GS",
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
    param!(&[0x00], 0x01, "REVERB CHARACTER"),
    param!(&[0x01], 0x01, "REVERB LEVEL"),
    param!(&[0x02], 0x01, "REVERB (DELAY) TIME"),
    param!(&[0x03], 0x01, "DELAY TIME"),
    param!(&[0x04], 0x01, "DELAY FEEDBACK"),
    param!(&[0x05], 0x01, "CHORUS LEVEL"),
    param!(&[0x06], 0x01, "CHORUS FEEDBACK"),
    param!(&[0x07], 0x01, "CHORUS DELAY"),
    param!(&[0x08], 0x01, "CHORUS RATE"),
    param!(&[0x09], 0x01, "CHORUS DEPTH"),
];

const SC_7_PAM_PATCH: ParameterAddressMap = &[
    param!(&[0x00], 0x01, "RX. CHANNEL"),
    param!(&[0x01], 0x01, "RX. NRPN"),
    param!(&[0x02], 0x01, "MOD LFO RATE CONTROL"),
    param!(&[0x03], 0x01, "MOD LFO PITCH DEPTH"),
    param!(&[0x04], 0x01, "CAF TVF CUT OFF CONTROL"),
    param!(&[0x05], 0x01, "CAF AMPLITUDE CONTROL"),
    param!(&[0x06], 0x01, "CAF LFO RATE CONTROL"),
    param!(&[0x07], 0x01, "CAF LFO PITCH DEPTH"),
];
