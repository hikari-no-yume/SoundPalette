//! Functions exported from the WebAssembly library and related FFI utilities
//! live here.

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

/// Return the number of "other events" in a [crate::midi::MidiData] returned
/// by [read_midi_and_log].
#[export_name = "SoundPalette_midi_data_other_events_len"]
pub unsafe extern "C" fn midi_data_other_events_len(
    midi_data: &mut crate::midi::MidiData,
) -> usize {
    midi_data.other_events.len()
}

/// Return the [crate::midi::AbsoluteTime] for an "other event", by index, in a
/// [crate::midi::MidiData] returned by [read_midi_and_log].
#[export_name = "SoundPalette_midi_data_other_events_get_absolute_time"]
pub unsafe extern "C" fn midi_data_other_events_get_absolute_time(
    midi_data: &mut crate::midi::MidiData,
    index: usize,
) -> crate::midi::AbsoluteTime {
    midi_data.other_events[index].0
}

/// Return the length of the byte data for an "other event", by index, in a
/// [crate::midi::MidiData] returned by [read_midi_and_log].
#[export_name = "SoundPalette_midi_data_other_events_get_bytes_len"]
pub unsafe extern "C" fn midi_data_other_events_get_bytes_len(
    midi_data: &mut crate::midi::MidiData,
    index: usize,
) -> usize {
    midi_data.other_events[index].1.len()
}

/// Return a pointer to the byte data for an "other event", by index, in a
/// [crate::midi::MidiData] returned by [read_midi_and_log].
#[export_name = "SoundPalette_midi_data_other_events_get_bytes_ptr"]
pub unsafe extern "C" fn midi_data_other_events_get_bytes_ptr(
    midi_data: &mut crate::midi::MidiData,
    index: usize,
) -> *const u8 {
    midi_data.other_events[index].1.as_ptr()
}

/// Return a string (which must be freed with [string_free]) containing the
/// interpretation of the byte data for an "other event", by index, in a
/// [crate::midi::MidiData] returned by [read_midi_and_log].
#[export_name = "SoundPalette_midi_data_other_events_get_bytes_interpretation"]
pub unsafe extern "C" fn midi_data_other_events_get_bytes_interpretation(
    midi_data: &mut crate::midi::MidiData,
    index: usize,
) -> *mut String {
    Box::leak(Box::new(format!(
        "{:?}",
        crate::sysex::parse_sysex(&midi_data.other_events[index].1)
    )))
}

/// Free a [crate::midi::MidiData] allocated by [read_midi_and_log].
#[export_name = "SoundPalette_midi_data_free"]
pub unsafe extern "C" fn midi_data_free(midi: *mut crate::midi::MidiData) {
    drop(Box::from_raw(midi))
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
