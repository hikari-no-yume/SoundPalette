/*
 * Part of SoundPalette by hikari_no_yume.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
// This crate will be called SoundPalette whether Rust likes it or not.
#![allow(non_snake_case)]
// These are internal interfaces and the safety properties are usually obvious.
#![allow(clippy::missing_safety_doc)]

pub mod midi;
pub mod sysex;
pub mod ui;
pub mod wasm_ffi;
