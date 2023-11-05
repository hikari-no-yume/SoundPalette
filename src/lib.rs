// This crate will be called SoundPalette whether Rust likes it or not.
#![allow(non_snake_case)]
// These are internal interfaces and the safety properties are usually obvious.
#![allow(clippy::missing_safety_doc)]

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

/// Format `bytes_len` bytes starting at `bytes_ptr` (e.g. from [bytes_new]) as
/// hexadecimal text. The text is appended to `string`, which must be allocated
/// with [string_new].
#[export_name = "SoundPalette_bytes_to_hex"]
pub unsafe extern "C" fn bytes_to_hex(string: &mut String, bytes_ptr: *const u8, bytes_len: usize) {
    let bytes = slice_for_bytes(bytes_ptr, bytes_len);

    for chunk in bytes.chunks(8) {
        for (byte_idx, &byte) in chunk.iter().enumerate() {
            use std::fmt::Write;
            write!(
                string,
                "{:02X}{}",
                byte,
                if byte_idx == chunk.len() - 1 {
                    '\n'
                } else {
                    ' '
                }
            )
            .unwrap();
        }
    }
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
