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
    table_stream.th(format_args!("Event"));
    table_stream.th(format_args!("Interpretation"));
    table_stream.end_tr();

    for (time, ref bytes) in &data.other_events {
        table_stream.td(format_args!("{}", time));
        table_stream.td(format_args!("{}", format_bytes(bytes)));
        match parse_sysex(bytes) {
            Ok(sysex) => table_stream.td(format_args!("SysEx: {}", sysex)),
            Err(err) => table_stream.td(format_args!("{:?}: {}", err, format_bytes(bytes))),
        }
        table_stream.end_tr();
    }
}
