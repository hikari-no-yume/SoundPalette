//! Roland GS.
//!
//! References:
//! - Roland SC-55 Owner's Manual.
//! - Roland SC-55mkII Owner's Manual.
//! - Roland SC-7 Owner's Manual (not a GS device, only has a tiny subset).

use super::ModelInfo;

/// Roland GS.
pub const GS: ModelInfo = ModelInfo {
    model_id: &[0x42],
    name: "Roland GS",
    default_device_id: 0x10, // SC-55 and SC-7 respond to this, at least
    address_size: 3,
    address_block_map: &[], // TODO: add at least the SC-7 stuff
};
