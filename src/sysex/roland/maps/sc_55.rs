/*
 * Part of SoundPalette by hikari_no_yume.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! Roland SC-55/SC-155.
//!
//! References:
//! - Roland SC-55 Owner's Manual.
//! - Roland SC-55mkII Owner's Manual.

use super::ModelInfo;

/// Roland SC-55/SC-155. This device also uses the GS model ID for some things.
pub const SC_55: ModelInfo = ModelInfo {
    model_id: &[0x45],
    name: "Roland SC-55/SC-155",
    default_device_id: 0x10,
    address_size: 3,
    address_block_map: &[], // TODO
};
