/*
 * Part of SoundPalette by hikari_no_yume.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! Functions exported from the WebAssembly library and related FFI utilities
//! live here.

/// Get pointer to the UTF-8 bytes of the version number string.
#[export_name = "SoundPalette_version_ptr"]
pub extern "C" fn version_ptr() -> *const u8 {
    crate::VERSION.as_ptr()
}

/// Get the number if UTF-8 bytes in the version number string.
#[export_name = "SoundPalette_version_len"]
pub extern "C" fn version_len() -> usize {
    crate::VERSION.len()
}

/// Allocate `size` bytes of memory and return a pointer to it. The caller
/// is responsible for releasing it with [bytes_free]. The memory is not
/// initialized.
#[export_name = "SoundPalette_bytes_new"]
pub extern "C" fn bytes_new(size: usize) -> *mut u8 {
    let mut vec = Vec::<u8>::with_capacity(size);
    let ptr = vec.as_mut_ptr();
    vec.leak();
    ptr
}

/// Free memory allocated with [bytes_new]. `size` must be the same size that
/// was passed to [bytes_new].
#[export_name = "SoundPalette_bytes_free"]
pub unsafe extern "C" fn bytes_free(ptr: *mut u8, size: usize) {
    drop(unsafe { Vec::from_raw_parts(ptr, size, size) });
}

#[cfg(test)]
/// Helper for accessing memory allocated with e.g. [bytes_new].
unsafe fn slice_for_bytes_mut<'a>(ptr: *mut u8, size: usize) -> &'a mut [u8] {
    std::slice::from_raw_parts_mut(ptr, size)
}

/// Helper for accessing memory allocated with e.g. [bytes_new].
unsafe fn slice_for_bytes<'a>(ptr: *const u8, size: usize) -> &'a [u8] {
    std::slice::from_raw_parts(ptr, size)
}

/// Allocate a [String] on the heap and return a pointer to it. This type is
/// opaque to non-Rust code and should only be acted on with exported Rust
/// functions. The caller is responsible for releasing it with [string_free].
#[export_name = "SoundPalette_string_new"]
pub extern "C" fn string_new(capacity: usize) -> *mut String {
    Box::leak(Box::new(String::with_capacity(capacity)))
}

/// Clear a [String] allocated by [string_new].
#[export_name = "SoundPalette_string_clear"]
pub unsafe extern "C" fn string_clear(string: &mut String) {
    string.clear()
}

/// Get a pointer to the UTF-8 bytes of a string allocated by [string_new].
/// Don't use the pointer to modify the bytes.
#[export_name = "SoundPalette_string_ptr"]
pub unsafe extern "C" fn string_ptr(string: &String) -> *const u8 {
    string.as_ptr()
}

/// Get the length (in UTF-8 bytes) of a string allocated by [string_new].
#[export_name = "SoundPalette_string_len"]
pub unsafe extern "C" fn string_len(string: &String) -> usize {
    string.len()
}

/// Free a [String] allocated by [string_new].
#[export_name = "SoundPalette_string_free"]
pub unsafe extern "C" fn string_free(string: *mut String) {
    drop(Box::from_raw(string))
}

/// Get a pointer to the bytes of a bytevec.
/// Don't use the pointer to modify the bytes.
#[export_name = "SoundPalette_bytevec_ptr"]
pub unsafe extern "C" fn bytevec_ptr(bytevec: &Vec<u8>) -> *const u8 {
    bytevec.as_ptr()
}

/// Get the length of a bytevec.
#[export_name = "SoundPalette_bytevec_len"]
pub unsafe extern "C" fn bytevec_len(bytevec: &Vec<u8>) -> usize {
    bytevec.len()
}

/// Free a bytevec.
#[export_name = "SoundPalette_bytevec_free"]
pub unsafe extern "C" fn bytevec_free(bytevec: *mut Vec<u8>) {
    drop(Box::from_raw(bytevec))
}

/// Read `bytes_len` bytes of Standard MIDI File data starting at `bytes_ptr`
/// and log the parsing by appending to `string`, which must have been allocated
/// with [string_new]. Returns a newly allocated pointer to MIDI data, which is
/// opaque to non-Rust code and must be freed with [midi_data_free].
#[export_name = "SoundPalette_read_midi_and_log"]
pub unsafe extern "C" fn read_midi_and_log(
    string: &mut String,
    bytes_ptr: *const u8,
    bytes_len: usize,
) -> *mut crate::midi::MidiData {
    use std::io::Cursor;

    let mut bytes = Cursor::new(slice_for_bytes(bytes_ptr, bytes_len));
    let mut log_tmp = Cursor::new(Vec::<u8>::new());

    match crate::midi::read_midi(&mut bytes, true, &mut log_tmp) {
        Ok(data) => {
            // FIXME: don't use this temporary extra buffer
            string.push_str(&String::from_utf8(log_tmp.into_inner()).unwrap());
            Box::leak(Box::new(data))
        }
        Err(e) => {
            use std::fmt::Write;
            writeln!(string, "Error: {:?}", e).unwrap();
            std::ptr::null_mut()
        }
    }
}

/// Create an empty MIDI file. Returns a newly allocated pointer to MIDI data,
/// which is opaque to non-Rust code and must be freed with [midi_data_free].
#[export_name = "SoundPalette_midi_data_new"]
pub unsafe extern "C" fn midi_data_new() -> *mut crate::midi::MidiData {
    Box::leak(Box::new(crate::midi::MidiData {
        // Something divisible by 10 is desirable, see midi_data_add_sysex.
        division: crate::midi::Division::TicksPerQuarterNote(120),
        channel_messages: Vec::new(),
        other_events: Vec::new(),
    }))
}

/// Outputs a table of "other events" from a [crate::midi::MidiData] returned
/// by [read_midi_and_log] or [midi_data_new]. The table is returned in
/// [crate::ui::NullTerminatedStringTableStream] format by appending it to a
/// string allocated with [string_new].
#[export_name = "SoundPalette_midi_data_list_other_events"]
pub unsafe extern "C" fn midi_data_list_other_events(
    string: &mut String,
    midi_data: &crate::midi::MidiData,
    with_time_and_kind: bool,
) {
    crate::ui::list_other_events(
        &mut crate::ui::NullTerminatedStringTableStream::new(string),
        midi_data,
        with_time_and_kind,
    )
}

/// Adds a SysEx (decoded from a string consisting of `in_sysex_len` UTF-8 bytes
/// starting at `in_sysex_bytes`) to a [crate::midi::MidiData] returned by
/// [midi_data_new]. If the SysEx can't be decoded, an error is appended to a
/// string allocated with [string_new] and [false] is returned.
#[export_name = "SoundPalette_midi_data_add_sysex"]
pub unsafe extern "C" fn midi_data_add_sysex(
    midi_data: &mut crate::midi::MidiData,
    out_string: &mut String,
    in_sysex_bytes: *const u8,
    in_sysex_len: usize,
) -> bool {
    let in_sysex = slice_for_bytes(in_sysex_bytes, in_sysex_len);
    let in_sysex = std::str::from_utf8(in_sysex).unwrap();

    let Ok(sysex_bytes) = crate::ui::decode_sysex(out_string, in_sysex) else {
        return false;
    };

    // SC-55mkII and SC-7 manuals both say a GM or GS reset takes about 50ms to
    // complete. Therefore, let's put 50ms between all SysEx messages.
    // TODO: Use shorter delay (40ms or 20ms as appropriate) when no reset is
    //       in the list.
    // Assumption: All existing events have been added by this function, so they
    //             are all in order, and there is no tempo or time signature
    //             meta event to change from the default of 120bpm, 4/4.
    let crate::midi::Division::TicksPerQuarterNote(ticks_per_quarter_note) = midi_data.division
    else {
        panic!();
    };
    let ticks_per_quarter_note: crate::midi::AbsoluteTime = ticks_per_quarter_note.into();
    let new_event_time = if let Some(&(last_event_time, _)) = midi_data.other_events.last() {
        last_event_time + ((ticks_per_quarter_note * 120) / 60).div_ceil(1000 / 50)
    } else {
        0
    };
    midi_data.other_events.push((new_event_time, sysex_bytes));
    true
}

/// Clears the "other events" from a [crate::midi::MidiData] returned by
/// [midi_data_new].
#[export_name = "SoundPalette_midi_data_clear_other_events"]
pub unsafe extern "C" fn midi_data_clear_other_events(midi_data: &mut crate::midi::MidiData) {
    midi_data.other_events.clear()
}

/// Create Standard MIDI File format 0 data from a [crate::midi::MidiData]
/// allocated by [midi_data_new]. Returns a bytevec that must be freed with
/// [bytevec_free].
#[export_name = "SoundPalette_midi_data_write_midi"]
pub unsafe extern "C" fn midi_data_write_midi(
    midi_data: &mut crate::midi::MidiData,
) -> *mut Vec<u8> {
    use std::io::Cursor;

    let mut midi_bytes = Vec::new();
    crate::midi::write_midi(
        &mut Cursor::new(&mut midi_bytes),
        midi_data,
        &mut std::io::empty(),
    )
    .unwrap();
    Box::leak(Box::new(midi_bytes))
}

/// Free a [crate::midi::MidiData] allocated by [read_midi_and_log] or
/// [midi_data_new].
#[export_name = "SoundPalette_midi_data_free"]
pub unsafe extern "C" fn midi_data_free(midi: *mut crate::midi::MidiData) {
    drop(Box::from_raw(midi))
}

/// Checks an ASCII SysEx string consisting of `in_sysex_len` UTF-8 bytes
/// starting at `in_sysex_bytes`, appending the result to a string.
#[export_name = "SoundPalette_check_sysex"]
pub unsafe extern "C" fn check_sysex(
    out_string: &mut String,
    in_sysex_bytes: *const u8,
    in_sysex_len: usize,
) {
    let in_sysex = slice_for_bytes(in_sysex_bytes, in_sysex_len);
    let in_sysex = std::str::from_utf8(in_sysex).unwrap();

    if let Ok(ref sysex_bytes) = crate::ui::decode_sysex(out_string, in_sysex) {
        crate::ui::check_sysex(out_string, sysex_bytes);
    }
}

pub struct SysExGeneratorMenuStack(crate::ui::MenuStack<Box<dyn crate::sysex::SysExGenerator>>);

/// Create [SysExGeneratorMenuStack].
#[export_name = "SoundPalette_sysex_generator_menu_stack_new"]
pub unsafe extern "C" fn sysex_generator_menu_stack_new() -> *mut SysExGeneratorMenuStack {
    Box::leak(Box::new(SysExGeneratorMenuStack(
        crate::ui::MenuStack::new(Box::new(crate::sysex::generate_sysex())),
    )))
}

/// List the current menu items of a [SysExGeneratorMenuStack] by appending them
/// to a string with null separation.
#[export_name = "SoundPalette_sysex_generator_menu_stack_list_items"]
pub unsafe extern "C" fn sysex_generator_menu_stack_list_items(
    out_string: &mut String,
    stack: &SysExGeneratorMenuStack,
) {
    stack.0.list_items_with_null_separation(out_string);
}

/// Descend in a [SysExGeneratorMenuStack]'s menu by item index, pushing the
/// result to its stack. If the result is a SysEx generator, it is immediately
/// popped from the stack, a SysEx is generated in hexadecimal form and appended
/// to the String, and [true] is returned. If the result is a new menu, [false]
/// is returned.
#[export_name = "SoundPalette_sysex_generator_menu_stack_push"]
pub unsafe extern "C" fn sysex_generator_menu_stack_push(
    out_string: &mut String,
    stack: &mut SysExGeneratorMenuStack,
    item_idx: usize,
) -> bool {
    let have_command = stack.0.push(item_idx);
    if have_command {
        let sysex_generator = stack.0.pop_command();
        let mut sysex_bytes = Vec::new();
        sysex_generator.generate(&mut sysex_bytes);

        use std::fmt::Write;
        write!(out_string, "{}", crate::midi::format_bytes(&sysex_bytes)).unwrap();
    }
    have_command
}

/// Pop the menu at the top of a [SysExGeneratorMenuStack]'s menu stack.
#[export_name = "SoundPalette_sysex_generator_menu_stack_pop"]
pub unsafe extern "C" fn sysex_generator_menu_stack_pop(stack: &mut SysExGeneratorMenuStack) {
    stack.0.pop_submenu();
}

/// Free a [SysExGeneratorMenuStack].
#[export_name = "SoundPalette_sysex_generator_menu_stack_free"]
pub unsafe extern "C" fn sysex_generator_menu_stack_free(stack: *mut SysExGeneratorMenuStack) {
    drop(Box::from_raw(stack))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bytes() {
        let size = 10_000_000;

        let ptr = bytes_new(size);

        let slice = unsafe { slice_for_bytes_mut(ptr, size) };
        slice.fill(7);

        let slice = unsafe { slice_for_bytes(ptr, size) };
        assert_eq!(slice.iter().map(|&n| n as usize).sum::<usize>(), 7 * size);

        unsafe { bytes_free(ptr, size) };
    }

    #[test]
    fn test_string() {
        let s = string_new(7);

        let cool_string = "hello, world";

        unsafe { &mut *s }.push_str(cool_string);

        assert_eq!(unsafe { string_len(&mut *s) }, cool_string.len());

        let slice = unsafe { slice_for_bytes(string_ptr(&mut *s), string_len(&mut *s)) };
        assert_eq!(slice, cool_string.as_bytes());

        unsafe { string_clear(&mut *s) };
        assert_eq!(unsafe { string_len(&mut *s) }, 0);

        unsafe { string_free(s) };
    }
}
