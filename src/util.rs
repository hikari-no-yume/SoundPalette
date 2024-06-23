/*
 * Part of SoundPalette by hikari_no_yume.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! MIDI utilities.

use crate::midi::{ChannelMessage as Message, ChannelMessageKind as Kind, MidiData};

pub fn convert_caf_to_cc(data: &mut MidiData, controller: u8, min: u8, max: u8) {
    for (_time, Message { channel: _, kind }) in &mut data.channel_messages {
        let &mut Kind::ChannelPressure(pressure) = kind else {
            continue;
        };
        let normalized_pressure = pressure as f32 / 127.0;
        let value = min as f32 + normalized_pressure * (max as f32 - min as f32);
        let value = value
            .floor()
            .clamp(min.min(max) as f32, max.max(min) as f32) as u8;
        *kind = Kind::ControlChange {
            control: controller,
            value,
        };
    }
}
