// This crate will be called SoundPalette whether Rust likes it or not.
#![allow(non_snake_case)]
// These are internal interfaces and the safety properties are usually obvious.
#![allow(clippy::missing_safety_doc)]

pub mod midi;
pub mod sysex;
pub mod wasm_ffi;
