/*
 * Part of SoundPalette by hikari_no_yume.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! Roland MT-32.
//!
//! References:
//! - Roland MT-32 Owner's Manual.

use super::ModelInfo;

/// Roland MT-32. Full support is unlikely for now, but it's good to have the
/// model ID recognised.
pub const MT_32: ModelInfo = ModelInfo {
    model_id: &[0x16],
    name: "MT-32",
    default_device_id: 0x10,
    address_size: 3,
    address_block_map: &[], // TODO
};
