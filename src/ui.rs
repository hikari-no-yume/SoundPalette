//! User interface things, especially those shared between the web app and CLI.

use crate::midi::{format_bytes, MidiData};
use crate::sysex::parse_sysex;
use std::fmt::Arguments;

// Utilities

/// Generic way to present a table of data (e.g. a list of MIDI events) to the
/// user.
pub trait TableStream {
    /// Output a cell for a table heading to the current row (HTML `<th>`).
    /// TODO: Actually use this for something? Currently, the first row is
    /// always a header, and other rows can't contain header cells.
    fn th(&mut self, c: Arguments);
    /// Output a normal cell to the current row (HTML `<td>`).
    fn td(&mut self, c: Arguments);
    /// End the current row (HTML `<tr>`). New cells will go in the next row.
    /// This must always be called after pushing the cells for the current row.
    fn end_tr(&mut self);
}

pub struct StderrTableStream {
    first_cell: bool,
}
impl StderrTableStream {
    #[allow(clippy::new_without_default)]
    pub fn new() -> StderrTableStream {
        StderrTableStream { first_cell: true }
    }
}
impl TableStream for StderrTableStream {
    fn th(&mut self, c: Arguments) {
        self.td(c)
    }
    fn td(&mut self, c: Arguments) {
        if self.first_cell {
            self.first_cell = false;
        } else {
            eprint!("\t");
        }
        eprint!("{}", c);
    }
    fn end_tr(&mut self) {
        eprintln!();
        self.first_cell = true;
    }
}

/// Makes a string like `"col1\0col2\0col3\0\0cell1\0cell2\0cell3\0\0"` that
/// can be easily yeeted across the FFI barrier and consumed by JavaScript.
pub struct NullTerminatedStringTableStream<'a> {
    string: &'a mut String,
}
impl NullTerminatedStringTableStream<'_> {
    pub fn new(string: &mut String) -> NullTerminatedStringTableStream {
        NullTerminatedStringTableStream { string }
    }
}
impl TableStream for NullTerminatedStringTableStream<'_> {
    fn th(&mut self, c: Arguments) {
        self.td(c)
    }
    fn td(&mut self, c: Arguments) {
        use std::fmt::Write;

        let old_len = self.string.len();
        write!(self.string, "{}", c).unwrap();
        // Ensure there weren't any unexpected null bytes added, and that the
        // cell isn't empty, since these are used for delimiting.
        assert!(self.string.len() != old_len);
        assert!(!self.string.as_bytes()[old_len..self.string.len()]
            .iter()
            .any(|&byte| byte == b'\0'));

        write!(self.string, "\0").unwrap();
    }
    fn end_tr(&mut self) {
        use std::fmt::Write;

        write!(self.string, "\0").unwrap();
    }
}

#[cfg(test)]
#[test]
fn test_null_terminated_table_stream() {
    let mut buf = String::new();
    let mut stream = NullTerminatedStringTableStream::new(&mut buf);
    stream.th(format_args!("foo"));
    stream.th(format_args!("bar"));
    stream.end_tr();
    stream.td(format_args!("foo1"));
    stream.td(format_args!("bar1"));
    stream.end_tr();
    stream.td(format_args!("foo2"));
    stream.td(format_args!("bar2"));
    stream.end_tr();
    assert_eq!(buf, "foo\0bar\0\0foo1\0bar1\0\0foo2\0bar2\0\0");
}

// UI entry-points

pub fn list_other_events(table_stream: &mut impl TableStream, data: &MidiData) {
    table_stream.th(format_args!("Time"));
    table_stream.th(format_args!("Event (raw)"));
    table_stream.th(format_args!("Kind"));
    table_stream.th(format_args!("Detail"));
    table_stream.end_tr();

    for (time, ref bytes) in &data.other_events {
        // Skip meta events.
        // TODO: Display at least text events, they're useful as comments.
        if bytes.first() == Some(&0xFF) {
            continue;
        }

        table_stream.td(format_args!("{}", time));
        table_stream.td(format_args!("{}", format_bytes(bytes)));
        match parse_sysex(bytes) {
            Ok(sysex) => {
                table_stream.td(format_args!("SysEx"));
                table_stream.td(format_args!("{}", sysex));
            }
            Err(err) => {
                table_stream.td(format_args!("{:?}", err));
                table_stream.td(format_args!("—"));
            }
        }
        table_stream.end_tr();
    }
}

pub fn check_sysex(out_string: &mut String, in_sysex: &str) {
    use std::fmt::Write;

    let mut sysex_bytes = Vec::with_capacity(in_sysex.len() / 2);
    for hex_byte in in_sysex.split_whitespace() {
        // hex suffix style used by SoundPalette
        let hex_byte = hex_byte.strip_suffix('h').unwrap_or(hex_byte);
        // hex suffix style used by MIDI spec, Roland and Yamaha
        let hex_byte = hex_byte.strip_suffix('H').unwrap_or(hex_byte);
        if hex_byte.len() != 2
            || !hex_byte.is_ascii()
            || !hex_byte.as_bytes()[0].is_ascii_hexdigit()
            || !hex_byte.as_bytes()[1].is_ascii_hexdigit()
        {
            write!(
                out_string,
                "Error: {:?} is not recognised as a hex byte",
                hex_byte
            )
            .unwrap();
            return;
        }

        sysex_bytes.push(u8::from_str_radix(hex_byte, 16).unwrap());
    }

    if sysex_bytes.first() != Some(&0xF0) || sysex_bytes.last() != Some(&0xF7) {
        write!(
            out_string,
            "Error: not a complete sysex, needs to start with F0h and end with F7h"
        )
        .unwrap();
        return;
    }

    if sysex_bytes[1..sysex_bytes.len() - 1]
        .iter()
        .any(|&byte| byte > 0x7F)
    {
        write!(
            out_string,
            "Error: contains invalid data bytes, out of range (> 7Fh)"
        )
        .unwrap();
        return;
    }

    match parse_sysex(&sysex_bytes) {
        Ok(sysex) => {
            write!(out_string, "SysEx: {}", sysex).unwrap();
        }
        Err(err) => {
            write!(out_string, "Error: {:?}", err).unwrap();
        }
    }
}
