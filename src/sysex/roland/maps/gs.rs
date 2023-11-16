//! Roland GS.
//!
//! References:
//! - Roland SC-55 Owner's Manual.
//! - Roland SC-55mkII Owner's Manual.
//! - Roland SC-7 Owner's Manual (not a GS device, only has a tiny subset).

use super::{
    param_enum, param_range, param_simple, AddressBlockMap, ModelInfo, ParameterAddressMap,
};

/// Roland GS.
pub const GS: ModelInfo = ModelInfo {
    model_id: &[0x42],
    name: "Roland GS",
    default_device_id: 0x10, // SC-55 and SC-7 respond to this, at least
    address_size: 3,
    address_block_map: GS_ABM,
};

const GS_ABM: AddressBlockMap = &[
    (&[0x40, 0x00], "System Parameters", GS_PAM_SYSTEM),
    (
        &[0x40, 0x01],
        "Patch Parameters, Patch common",
        GS_PAM_PATCH_COMMON,
    ),
    // TODO: per-patch parameters, Drum setup parameters, Bulk dump
];

const GS_PAM_SYSTEM: ParameterAddressMap = &[
    // TODO: MASTER TUNE ("nibblized data" support missing)
    param_simple(&[0x04], 0x01, "MASTER VOLUME", None),
    param_range(
        &[0x05],
        0x01,
        "MASTER KEY-SHIFT",
        0x28..=0x58,
        Some(0x40),
        -24.0..=24.0,
        "semitones",
    ),
    // TODO: how to accomodate zero value for panning? There is no "unit".
    param_simple(&[0x06], 0x01, "MASTER PAN", Some(0x01..=0x7F)),
    param_enum(
        &[0x7F],
        0x01,
        // SC-55mkII name. Called "RESET TO THE GSstandard MODE" in the SC-55
        // manual.
        "MODE SET",
        // SC-55mkII manual also mentions 0x7F as valid data, but it does not
        // explain what it would do, and "MODE SET" is marked as "(Rx. only)",
        // so it can't be for querying. Perhaps that extra byte was an error,
        // it isn't in the original SC-55 manual.
        0x00..=0x00,
        &[(&[0x00], "GS Reset")],
    ),
];

const GS_PAM_PATCH_COMMON: ParameterAddressMap = &[
    // TODO: Patch Name (non-single-byte parameter support missing)
    // TODO: Voice Reserve (non-single-byte parameter support missing)
    param_enum(
        &[0x30],
        0x01,
        "REVERB MACRO",
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
    param_simple(&[0x31], 0x01, "REVERB CHARACTER", Some(0x00..=0x07)),
    param_simple(&[0x32], 0x01, "REVERB PRE-LPF", Some(0x00..=0x07)),
    param_simple(&[0x33], 0x01, "REVERB LEVEL", None),
    param_simple(&[0x34], 0x01, "REVERB TIME", None),
    param_simple(&[0x35], 0x01, "REVERB DELAY FEEDBACK", None),
    param_simple(&[0x36], 0x01, "REVERB SEND LEVEL TO CHORUS", None),
    // 37h is unoccupied!
    param_enum(
        &[0x38],
        0x01,
        "CHORUS MACRO",
        0x00..=0x07,
        &[
            (&[0x00], "Chorus 1"),
            (&[0x01], "Chorus 2"),
            (&[0x02], "Chorus 3"),
            (&[0x03], "Chorus 4"),
            (&[0x04], "Feedback Chorus"),
            (&[0x05], "Flanger"),
            (&[0x06], "Short Delay"),
            (&[0x07], "Short Delay (FB)"),
        ],
    ),
    param_simple(&[0x39], 0x01, "CHORUS PRE-LPF", Some(0x00..=0x07)),
    param_simple(&[0x3A], 0x01, "CHORUS LEVEL", None),
    param_simple(&[0x3B], 0x01, "CHORUS FEEDBACK", None),
    param_simple(&[0x3C], 0x01, "CHORUS DELAY", None),
    param_simple(&[0x3D], 0x01, "CHORUS RATE", None),
    param_simple(&[0x3E], 0x01, "CHORUS DEPTH", None),
    param_simple(&[0x3F], 0x01, "CHORUS SEND LEVEL TO REVERB", None),
];
