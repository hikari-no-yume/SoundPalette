//! Roland GS.
//!
//! References:
//! - Roland SC-55 Owner's Manual.
//! - Roland SC-55mkII Owner's Manual.
//! - Roland SC-7 Owner's Manual (not a GS device, only has a tiny subset).

use super::{
    param_range, param_simple, AddressBlockMap, ModelInfo, ParameterAddressMap,
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
    // TODO: Patch parameters, Drum setup parameters, Bulk dump
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
    // TODO: non-SC-7 parameters
];

// TODO: Voice Reserve (non-single-byte parameter support missing)
