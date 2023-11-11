//! Various Parameter Address Maps.
//!
//! TODO: These should probably be stored as data files?

use super::{
    AddressBlockMap, AddressBlockMapMap, Parameter, ParameterAddressMap, MD_ID_ROLAND_GS,
    MD_ID_ROLAND_SC_7,
};

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

pub const ADDRESS_BLOCK_MAP_MAP: AddressBlockMapMap =
    &[(MD_ID_ROLAND_SC_7, SC_7_ABM), (MD_ID_ROLAND_GS, GS_ABM)];

/// Roland SC-7 parameters.
const SC_7_ABM: AddressBlockMap = &[
    (
        0x00,
        0x00,
        "System parameters, Effect Control",
        SC_7_PAM_SYSTEM,
    ),
    (
        0x01,
        0x00,
        "Patch parameters, Patch block 0",
        SC_7_PAM_PATCH,
    ),
    (
        0x01,
        0x01,
        "Patch parameters, Patch block 1",
        SC_7_PAM_PATCH,
    ),
    (
        0x01,
        0x02,
        "Patch parameters, Patch block 2",
        SC_7_PAM_PATCH,
    ),
    (
        0x01,
        0x03,
        "Patch parameters, Patch block 3",
        SC_7_PAM_PATCH,
    ),
    (
        0x01,
        0x04,
        "Patch parameters, Patch block 4",
        SC_7_PAM_PATCH,
    ),
    (
        0x01,
        0x05,
        "Patch parameters, Patch block 5",
        SC_7_PAM_PATCH,
    ),
    (
        0x01,
        0x06,
        "Patch parameters, Patch block 6",
        SC_7_PAM_PATCH,
    ),
    (
        0x01,
        0x07,
        "Patch parameters, Patch block 7",
        SC_7_PAM_PATCH,
    ),
    (
        0x01,
        0x08,
        "Patch parameters, Patch block 8",
        SC_7_PAM_PATCH,
    ),
    (
        0x01,
        0x09,
        "Patch parameters, Patch block 9",
        SC_7_PAM_PATCH,
    ),
    (
        0x01,
        0x0A,
        "Patch parameters, Patch block A",
        SC_7_PAM_PATCH,
    ),
    (
        0x01,
        0x0B,
        "Patch parameters, Patch block B",
        SC_7_PAM_PATCH,
    ),
    (
        0x01,
        0x0C,
        "Patch parameters, Patch block C",
        SC_7_PAM_PATCH,
    ),
    (
        0x01,
        0x0D,
        "Patch parameters, Patch block D",
        SC_7_PAM_PATCH,
    ),
    (
        0x01,
        0x0E,
        "Patch parameters, Patch block E",
        SC_7_PAM_PATCH,
    ),
    (
        0x01,
        0x0F,
        "Patch parameters, Patch block F",
        SC_7_PAM_PATCH,
    ),
];

const SC_7_PAM_SYSTEM: ParameterAddressMap = &[
    param!(0x00, 0x01, "REVERB CHARACTER"),
    param!(0x01, 0x01, "REVERB LEVEL"),
    param!(0x02, 0x01, "REVERB (DELAY) TIME"),
    param!(0x03, 0x01, "DELAY TIME"),
    param!(0x04, 0x01, "DELAY FEEDBACK"),
    param!(0x05, 0x01, "CHORUS LEVEL"),
    param!(0x06, 0x01, "CHORUS FEEDBACK"),
    param!(0x07, 0x01, "CHORUS DELAY"),
    param!(0x08, 0x01, "CHORUS RATE"),
    param!(0x09, 0x01, "CHORUS DEPTH"),
];

const SC_7_PAM_PATCH: ParameterAddressMap = &[
    param!(0x00, 0x01, "RX. CHANNEL"),
    param!(0x01, 0x01, "RX. NRPN"),
    param!(0x02, 0x01, "MOD LFO RATE CONTROL"),
    param!(0x03, 0x01, "MOD LFO PITCH DEPTH"),
    param!(0x04, 0x01, "CAF TVF CUT OFF CONTROL"),
    param!(0x05, 0x01, "CAF AMPLITUDE CONTROL"),
    param!(0x06, 0x01, "CAF LFO RATE CONTROL"),
    param!(0x07, 0x01, "CAF LFO PITCH DEPTH"),
];

/// Roland GS parameters.
const GS_ABM: AddressBlockMap = &[
    // TODO: add the SC-7 subset at least.
];
