//! Roland SC-7.
//!
//! Reference: Roland SC-7 Owner's Manual.

use super::{
    param_enum, param_range, param_simple, AddressBlockMap, ModelInfo, ParameterAddressMap,
};

/// Roland SC-7. This device also uses the GS model ID for some things.
pub const SC_7: ModelInfo = ModelInfo {
    model_id: &[0x56],
    name: "Roland SC-7",
    default_device_id: 0x10, // non-configurable
    address_size: 3,
    address_block_map: SC_7_ABM,
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
    param_range(
        &[0x02],
        0x01,
        "MOD LFO RATE CONTROL",
        0x00..=0x7F,
        Some(0x40),
        -10.0..=10.0,
        "Hz",
    ),
    param_range(
        &[0x03],
        0x01,
        "MOD LFO PITCH DEPTH",
        0x00..=0x7F,
        None,
        0.0..=600.0,
        "cents",
    ),
    // Unit not specified in SC-7 manual, but the SC-55 has what seems to be the
    // same control (same name, same range, same function) and it says cents.
    param_range(
        &[0x04],
        0x01,
        "CAF TVF CUT OFF CONTROL",
        0x00..=0x7F,
        Some(0x40),
        -9600.0..=9600.0,
        "cents",
    ),
    param_range(
        &[0x05],
        0x01,
        "CAF AMPLITUDE CONTROL",
        0x00..=0x7F,
        Some(0x40),
        -100.0..=100.0,
        "%",
    ),
    param_range(
        &[0x06],
        0x01,
        "CAF LFO RATE CONTROL",
        0x00..=0x7F,
        Some(0x40),
        -10.0..=10.0,
        "Hz",
    ),
    param_range(
        &[0x07],
        0x01,
        "CAF LFO PITCH DEPTH",
        0x00..=0x7F,
        None,
        0.0..=600.0,
        "Hz",
    ),
];
