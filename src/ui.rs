//! User interface things, especially those shared between the web app and CLI.

use crate::midi::{format_bytes, MidiData};
use crate::sysex::parse_sysex;
use std::fmt::{Arguments, Debug, Result as FmtResult};

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

/// Generic way to provide a menu hierarchy that ultimately leads to some kind
/// of useful command or other result (`T`).
///
/// The list of items in the menu must not change for the lifetime of the
/// [Menu].
pub trait Menu<T: Debug> {
    /// Get the number of items in the menu. All indices in the range
    /// `0..Menu::items_count()` must be valid arguments to [Menu::item_label]
    /// [Menu::item_disabled], and (if not disabled) [Menu::item_descend].
    fn items_count(&self) -> usize;

    /// Writes the label for an item to a provided destination.
    fn item_label(&self, item_idx: usize, write_to: &mut dyn std::fmt::Write) -> FmtResult;

    /// Returns [true] if an item is to be presented as "disabled" and therefore
    /// must not be used as an argument to [Menu::item_descend]. What happens if
    /// this is ignored is undefined.
    fn item_disabled(&self, item_idx: usize) -> bool {
        let _ = item_idx;
        false
    }

    /// Select a menu item by its index in the list (counting from 0). See
    /// return type for more detail. Calling this method must not, by itself,
    /// alter any state or perform any action.
    fn item_descend(&self, item_idx: usize) -> MenuItemResult<T>;
}

/// Result of calling [Menu::item_descend].
pub enum MenuItemResult<T: Debug> {
    /// Selecting the menu item opens a submenu.
    Submenu(Box<dyn Menu<T>>),
    /// Selecting the menu item leads to the command `T`.
    Command(T),
}

/// Print a menu hierarchy. This is a debugging tool.
pub fn print_menu<T, F>(menu: &dyn Menu<T>, with_command: &F)
where
    T: Debug,
    F: Fn(T),
{
    print_menu_inner(0, menu, with_command);

    fn print_menu_inner<T, F>(indent: usize, menu: &dyn Menu<T>, with_command: &F)
    where
        T: Debug,
        F: Fn(T),
    {
        for i in 0..menu.items_count() {
            for _ in 0..indent {
                eprint!("  ");
            }
            eprint!("* ");
            // Rust why can't I std::fmt::Write to an std::io::Write, this sucks
            let mut label = String::new();
            menu.item_label(i, &mut label).unwrap();
            eprint!("{}", label);
            if menu.item_disabled(i) {
                eprintln!(" (disabled)");
                continue;
            }
            match menu.item_descend(i) {
                MenuItemResult::Submenu(menu) => {
                    eprintln!();
                    print_menu_inner(indent + 1, &*menu, with_command);
                }
                MenuItemResult::Command(command) => {
                    eprint!(" => ");
                    with_command(command);
                    eprintln!();
                }
            }
        }
    }
}

/// A stack used for stateful tracking of the path taken through a hierarchy of
/// menus. The design is intended to simplify communication between the web UI
/// JS and the Rust library.
pub struct MenuStack<T: Debug> {
    stack: Vec<Box<dyn Menu<T>>>,
    command: Option<T>,
}
impl<T: Debug> MenuStack<T> {
    /// Start menu tracking at `root_menu`.
    pub fn new(root_menu: Box<dyn Menu<T>>) -> MenuStack<T> {
        MenuStack {
            stack: vec![root_menu],
            command: None,
        }
    }

    fn current_menu(&self) -> &dyn Menu<T> {
        assert!(self.command.is_none(), "Top of stack is not a menu!");
        &**self.stack.last().unwrap()
    }

    /// List the menu items for the menu at the top of the stack by writing them
    /// to a string, separated by nulls. Panics if the top of the stack is not
    /// a menu. Disabled items are represented by prefixing with ASCII control
    /// character "Cancel" (`'\x18'`).
    pub fn list_items_with_null_separation(&self, string: &mut String) {
        use std::fmt::Write;

        let current_menu = self.current_menu();
        let count = current_menu.items_count();
        for i in 0..count {
            if current_menu.item_disabled(i) {
                write!(string, "\x18").unwrap();
            }
            let old_len = string.len();
            current_menu.item_label(i, string).unwrap();
            // Ensure there weren't any unexpected null or Cancel bytes added.
            assert!(!string.as_bytes()[old_len..string.len()]
                .iter()
                .any(|&byte| byte == b'\0' || byte == b'\x18'));
            if i != count - 1 {
                write!(string, "\0").unwrap();
            }
        }
    }

    /// Select a menu item by index, pushing its submenu or command to the top
    /// of the stack. Panics if the top of the stack is not a menu.
    /// Result is the same as [MenuStack::have_command] and reflects the new
    /// state of the stack.
    pub fn push(&mut self, item_idx: usize) -> bool {
        match self.current_menu().item_descend(item_idx) {
            MenuItemResult::Submenu(menu) => self.stack.push(menu),
            MenuItemResult::Command(command) => self.command = Some(command),
        }

        self.have_command()
    }

    /// Returns [true] if the top of the stack is a command, and [false] if it
    /// is a menu.
    pub fn have_command(&self) -> bool {
        self.command.is_some()
    }

    /// Pop the submenu at the top of the stack. Panics if the top of the stack
    /// is not a menu, or if this is the root menu.
    pub fn pop_submenu(&mut self) {
        assert!(self.command.is_none(), "Top of stack is not a menu!");
        assert!(self.stack.len() != 1, "This is the root menu!");
        self.stack.pop();
    }

    /// Pops and returns the command at the top of the stack. Panics if the top
    /// of the stack is not a command.
    pub fn pop_command(&mut self) -> T {
        self.command.take().unwrap()
    }
}

#[cfg(test)]
#[test]
fn test_menu_stack() {
    let mut stack = MenuStack::new(Box::new(crate::sysex::generate_sysex()));
    let mut string = String::new();

    assert!(!stack.have_command());
    stack.list_items_with_null_separation(&mut string);
    assert_eq!(
        string.split_once('\0').unwrap().0,
        "Universal Non-Real Time (7Eh)"
    );
    string.clear();
    stack.push(0);

    assert!(!stack.have_command());
    stack.list_items_with_null_separation(&mut string);
    assert_eq!(string, "09h — General MIDI (@ Broadcast)");
    string.clear();
    stack.push(0);

    assert!(!stack.have_command());
    stack.list_items_with_null_separation(&mut string);
    assert_eq!(string, "01h — General MIDI System On");
    string.clear();
    stack.push(0);

    assert!(stack.have_command());
    stack.pop_command();

    assert!(!stack.have_command());
    stack.pop_submenu();

    stack.push(0);

    assert!(!stack.have_command());
    stack.list_items_with_null_separation(&mut string);
    assert_eq!(string, "01h — General MIDI System On");
    string.clear();
    stack.push(0);

    assert!(stack.have_command());
    let command = stack.pop_command();
    let mut vec = Vec::new();
    command.generate(&mut vec);
    assert_eq!(vec, &[0xF0, 0x7E, 0x7F, 0x09, 0x01, 0xF7]);
}

// UI entry-points

pub fn list_other_events(
    table_stream: &mut impl TableStream,
    data: &MidiData,
    with_time_and_kind: bool,
) {
    if with_time_and_kind {
        table_stream.th(format_args!("Time"));
    }
    table_stream.th(format_args!("Event (raw)"));
    if with_time_and_kind {
        table_stream.th(format_args!("Kind"));
    }
    table_stream.th(format_args!("Detail"));
    table_stream.end_tr();

    for (time, ref bytes) in &data.other_events {
        // Skip meta events.
        // TODO: Display at least text events, they're useful as comments.
        if bytes.first() == Some(&0xFF) {
            continue;
        }

        if with_time_and_kind {
            table_stream.td(format_args!("{}", time));
        }
        table_stream.td(format_args!("{}", format_bytes(bytes)));
        match parse_sysex(bytes) {
            Ok(sysex) => {
                if with_time_and_kind {
                    table_stream.td(format_args!("SysEx"));
                }
                table_stream.td(format_args!("{}", sysex));
            }
            Err(err) => {
                if with_time_and_kind {
                    table_stream.td(format_args!("{:?}", err));
                }
                table_stream.td(format_args!("—"));
            }
        }
        table_stream.end_tr();
    }
}

#[allow(clippy::result_unit_err)]
pub fn decode_sysex(out_string: &mut String, in_sysex: &str) -> Result<Vec<u8>, ()> {
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
            return Err(());
        }

        sysex_bytes.push(u8::from_str_radix(hex_byte, 16).unwrap());
    }

    if sysex_bytes.first() != Some(&0xF0) || sysex_bytes.last() != Some(&0xF7) {
        write!(
            out_string,
            "Error: not a complete sysex, needs to start with F0h and end with F7h"
        )
        .unwrap();
        return Err(());
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
        return Err(());
    }

    Ok(sysex_bytes)
}

pub fn check_sysex(out_string: &mut String, sysex_bytes: &[u8]) {
    use std::fmt::Write;

    match parse_sysex(sysex_bytes) {
        Ok(sysex) => {
            write!(out_string, "SysEx: {}", sysex).unwrap();
        }
        Err(err) => {
            write!(out_string, "Error: {:?}", err).unwrap();
        }
    }
}
