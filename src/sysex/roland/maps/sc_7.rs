/*
 * Part of SoundPalette by hikari_no_yume.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! Roland SC-7.
//!
//! Reference: Roland SC-7 Owner's Manual.

use super::{
    param_bool, param_enum, param_range, param_unsigned, AddressBlockMap, ModelInfo,
    ParameterAddressMap,
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
    (&[0x01, 0x00], "Patch parameters, Part 10", SC_7_PAM_PATCH),
    (&[0x01, 0x01], "Patch parameters, Part 1", SC_7_PAM_PATCH),
    (&[0x01, 0x02], "Patch parameters, Part 2", SC_7_PAM_PATCH),
    (&[0x01, 0x03], "Patch parameters, Part 3", SC_7_PAM_PATCH),
    (&[0x01, 0x04], "Patch parameters, Part 4", SC_7_PAM_PATCH),
    (&[0x01, 0x05], "Patch parameters, Part 5", SC_7_PAM_PATCH),
    (&[0x01, 0x06], "Patch parameters, Part 6", SC_7_PAM_PATCH),
    (&[0x01, 0x07], "Patch parameters, Part 7", SC_7_PAM_PATCH),
    (&[0x01, 0x08], "Patch parameters, Part 8", SC_7_PAM_PATCH),
    (&[0x01, 0x09], "Patch parameters, Part 9", SC_7_PAM_PATCH),
    (&[0x01, 0x0A], "Patch parameters, Part 11", SC_7_PAM_PATCH),
    (&[0x01, 0x0B], "Patch parameters, Part 12", SC_7_PAM_PATCH),
    (&[0x01, 0x0C], "Patch parameters, Part 13", SC_7_PAM_PATCH),
    (&[0x01, 0x0D], "Patch parameters, Part 14", SC_7_PAM_PATCH),
    (&[0x01, 0x0E], "Patch parameters, Part 15", SC_7_PAM_PATCH),
    (&[0x01, 0x0F], "Patch parameters, Part 16", SC_7_PAM_PATCH),
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
    param_unsigned(&[0x01], 0x01, "REVERB LEVEL", 0x00..=0x7F),
    param_unsigned(&[0x02], 0x01, "REVERB (DELAY) TIME", 0x00..=0x7F),
    param_unsigned(&[0x03], 0x01, "DELAY TIME", 0x00..=0x7F),
    param_unsigned(&[0x04], 0x01, "DELAY FEEDBACK", 0x00..=0x7F),
    param_unsigned(&[0x05], 0x01, "CHORUS LEVEL", 0x00..=0x7F),
    param_unsigned(&[0x06], 0x01, "CHORUS FEEDBACK", 0x00..=0x7F),
    param_unsigned(&[0x07], 0x01, "CHORUS DELAY", 0x00..=0x7F),
    param_unsigned(&[0x08], 0x01, "CHORUS RATE", 0x00..=0x7F),
    param_unsigned(&[0x09], 0x01, "CHORUS DEPTH", 0x00..=0x7F),
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
    param_bool(&[0x01], "RX. NRPN"),
    param_range(
        &[0x02],
        0x01,
        "MOD LFO RATE CONTROL",
        0x00..=0x7F,
        0x40,
        -10.0..=10.0,
        "Hz",
    ),
    param_range(
        &[0x03],
        0x01,
        "MOD LFO PITCH DEPTH",
        0x00..=0x7F,
        0x00,
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
        0x40,
        -9600.0..=9600.0,
        "cents",
    ),
    param_range(
        &[0x05],
        0x01,
        "CAF AMPLITUDE CONTROL",
        0x00..=0x7F,
        0x40,
        -100.0..=100.0,
        "%",
    ),
    param_range(
        &[0x06],
        0x01,
        "CAF LFO RATE CONTROL",
        0x00..=0x7F,
        0x40,
        -10.0..=10.0,
        "Hz",
    ),
    param_range(
        &[0x07],
        0x01,
        "CAF LFO PITCH DEPTH",
        0x00..=0x7F,
        0x00,
        0.0..=600.0,
        "Hz",
    ),
];
