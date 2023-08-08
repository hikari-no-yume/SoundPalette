use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::path::PathBuf;

macro_rules! logif {
    ($verbose:ident, $($arg:tt)+) => {
        if $verbose {
            eprintln!($($arg)*);
        }
    }
}

#[derive(Debug)]
enum Division {
    TicksPerQuarterNote(u16),
    TicksPerFrame {
        frame_rate: SMPTEFormat,
        ticks_per_frame: u8,
    },
}

#[derive(Debug)]
#[repr(i8)]
enum SMPTEFormat {
    /// 24fps
    SMPTEFormat24 = -24,
    /// 25fps
    SMPTEFormat25 = -25,
    /// 29.97fps (NTSC)
    SMPTEFormat29 = -29,
    /// 30fps
    SMPTEFormat30 = -30,
}
impl TryFrom<i8> for SMPTEFormat {
    type Error = ();
    fn try_from(byte: i8) -> Result<SMPTEFormat, ()> {
        match byte {
            -24 => Ok(Self::SMPTEFormat24),
            -25 => Ok(Self::SMPTEFormat25),
            -29 => Ok(Self::SMPTEFormat29),
            -30 => Ok(Self::SMPTEFormat30),
            _ => Err(()),
        }
    }
}

/// The ticks-per-quarter-note in the header is 15 bits, so with 32 bits for an
/// absolute tick counter, there can be (1 << 17) quarter notes, which seems
/// like plenty. There's even more room with ticks-per-frame!
type AbsoluteTime = u32;

#[derive(Debug)]
struct MidiData {
    division: Division,
    /// `u32` is an absolute timestamp.
    channel_messages: Vec<(AbsoluteTime, ChannelMessage)>,
    /// `u32` is an absolute timestamp. The bytes are a SysEx or meta event in
    /// SMF format, but with the length quantity removed.
    other_events: Vec<(AbsoluteTime, Vec<u8>)>,
}

#[derive(Debug)]
struct ChannelMessage {
    channel: u8,
    kind: ChannelMessageKind,
}

#[derive(Debug)]
#[repr(u8)]
enum ChannelMessageKind {
    NoteOff {
        key: u8,
        velocity: u8,
    } = 0x8,
    NoteOn {
        key: u8,
        velocity: u8,
    } = 0x9,
    PolyKeyPressure {
        key: u8,
        pressure: u8,
    } = 0xA,
    /// Also used for Channel Mode.
    ControlChange {
        control: u8,
        value: u8,
    } = 0xB,
    ProgramChange(u8) = 0xC,
    ChannelPressure(u8) = 0xD,
    PitchBendChange(u16) = 0xE,
}
impl ChannelMessageKind {
    fn discriminant(&self) -> u8 {
        // I wish Rust let me access the discriminant for an enum with fields
        // without unsafe code :(
        // https://doc.rust-lang.org/std/mem/fn.discriminant.html#accessing-the-numeric-value-of-the-discriminant
        unsafe { *<*const _>::from(self).cast::<u8>() }
    }
}

fn read_midi(path: PathBuf, v: bool) -> Result<MidiData, Box<dyn Error>> {
    let mut file = BufReader::new(File::open(path)?);

    // Read header chunk

    let header_4cc: [u8; 4] = read_bytes(&mut file)?;
    if header_4cc != *b"MThd" {
        return Err("File is not in Standard MIDI File format".into());
    }
    let header_len = read_u32(&mut file)?;
    if header_len < 6 {
        return Err("Header is too short".into());
    }

    let format = read_u16(&mut file)?;
    match format {
        // One song on one track
        0 => eprintln!("Reading MIDI file (Standard MIDI File format 0)."),
        // One song across several tracks
        1 => eprintln!("Reading MIDI file (Standard MIDI File format 1)."),
        // Several songs, each on a single track - they'd be mashed together
        // by this tool, which is bad!
        2 => return Err("Standard MIDI File format 2 is not supported".into()),
        // Those three are the only ones in the standard
        _ => return Err("Unknown Standard MIDI File format (not 0, 1 or 2)".into()),
    }

    let ntrks = read_u16(&mut file)?;
    if ntrks == 0 {
        return Err("MIDI file has no tracks!".into());
    } else if ntrks > 1 && format == 0 {
        return Err("Multiple tracks in a Standard MIDI File format 0 file".into());
    }
    eprintln!("Track count: {}", ntrks);

    let division = read_u16(&mut file)?;
    let division = match division >> 15 {
        0 => Division::TicksPerQuarterNote(division),
        1 => Division::TicksPerFrame {
            frame_rate: ((division >> 8) as i8)
                .try_into()
                .map_err(|_| "Unrecognized SMPTE format")?,
            ticks_per_frame: division as u8,
        },
        _ => unreachable!(),
    };
    eprintln!("Division: {:?}", division);

    // Read track chunks

    let mut channel_messages = Vec::new();
    let mut other_events = Vec::new();

    let mut trk = 0;
    while trk < ntrks {
        let chunk_4cc: [u8; 4] = read_bytes(&mut file)?;
        let chunk_len = read_u32(&mut file)?;

        // TODO: do we even need to skip other chunks?

        if chunk_4cc != *b"MTrk" {
            eprintln!(
                "Skipping unknown chunk type {:?} ({} bytes)",
                chunk_4cc, chunk_len
            );
            file.seek_relative(chunk_len.into())?;
            continue;
        }

        trk += 1;
        logif!(v, "Track {} ({} bytes):", trk, chunk_len);

        // The ticks-per-quarter-note in the header is 15 bits, so with 32 bits
        // for an absolute tick counter, there can be (1 << 17) quarter notes,
        // which seems like plenty. There's even more room with ticks-per-frame.
        let mut time: AbsoluteTime = 0;
        let mut bytes_left = chunk_len;

        // Read events

        let mut running_status = None;
        while bytes_left > 0 {
            let delta_time = read_variable_length_quantity_within(&mut file, &mut bytes_left)?;
            if delta_time > 0 {
                logif!(v, "Delta time: +{} ticks", delta_time);
                time = time
                    .checked_add(delta_time)
                    .ok_or("Song is too long (more than 4,294,967,295 ticks)")?;
            }
            // SMF steals encoding space from the status bytes for various MIDI
            // system messages that it doesn't want to be directly encodable.
            // Each event data can only be one of these SMF-defined events, or a
            // MIDI channel message.
            let first_byte = read_byte_within(&mut file, &mut bytes_left)?;
            match first_byte {
                0xF0 => {
                    running_status = None;
                    let length = read_variable_length_quantity_within(&mut file, &mut bytes_left)?;
                    logif!(v, "SysEx start ({} bytes)", length);
                    let mut bytes = vec![first_byte];
                    for _ in 0..length {
                        bytes.push(read_byte_within(&mut file, &mut bytes_left)?);
                    }
                    other_events.push((time, bytes));
                }
                0xF7 => {
                    running_status = None;
                    let length = read_variable_length_quantity_within(&mut file, &mut bytes_left)?;
                    logif!(v, "SysEx continuation ({} bytes)", length);
                    let mut bytes = vec![first_byte];
                    for _ in 0..length {
                        bytes.push(read_byte_within(&mut file, &mut bytes_left)?);
                    }
                    other_events.push((time, bytes));
                }
                0xFF => {
                    running_status = None;
                    let type_ = read_byte_within(&mut file, &mut bytes_left)?;
                    if type_ >= 128 {
                        return Err("Invalid meta event type".into());
                    }
                    let length = read_variable_length_quantity_within(&mut file, &mut bytes_left)?;
                    logif!(v, "Meta event type {:02X} ({} bytes)", type_, length);
                    let mut bytes = vec![first_byte, type_];
                    for _ in 0..length {
                        bytes.push(read_byte_within(&mut file, &mut bytes_left)?);
                    }
                    other_events.push((time, bytes));
                    if type_ == 0x2F {
                        logif!(v, "End of track.");
                    }
                }
                _ => {
                    // This is a MIDI channel message. It may begin with a
                    // status byte to change the message kind and channel, or it
                    // may omit it (Running Status). The remaining bytes are
                    // always the data bytes, which depend on the kind.
                    let (status, first_data_byte) = if first_byte & 0x80 != 0 {
                        logif!(
                            v,
                            "Status byte: channel {}, message kind {:X}",
                            first_byte & 0xf,
                            first_byte >> 4
                        );
                        running_status = Some(first_byte);
                        let first_data = read_byte_within(&mut file, &mut bytes_left)?;
                        (first_byte, first_data)
                    } else {
                        let status =
                            running_status.ok_or("Missing status byte in MIDI channel message")?;
                        (status, first_byte)
                    };
                    let message =
                        read_message_within(&mut file, &mut bytes_left, status, first_data_byte)?;
                    logif!(v, "{:?}", message);
                    channel_messages.push((time, message));
                }
            }
        }
    }

    eprintln!("Reached final track, done reading MIDI file.");

    Ok(MidiData {
        division,
        channel_messages,
        other_events,
    })
}

fn read_message_within<R: Read>(
    reader: &mut R,
    within: &mut u32,
    status: u8,
    first_data_byte: u8,
) -> Result<ChannelMessage, Box<dyn Error>> {
    let channel = status & 0xf;
    let kind = match status >> 4 {
        0x8 => {
            let velocity = read_byte_within(reader, within)?;
            ChannelMessageKind::NoteOff {
                key: first_data_byte,
                velocity,
            }
        }
        0x9 => {
            let velocity = read_byte_within(reader, within)?;
            ChannelMessageKind::NoteOn {
                key: first_data_byte,
                velocity,
            }
        }
        0xA => {
            let pressure = read_byte_within(reader, within)?;
            ChannelMessageKind::PolyKeyPressure {
                key: first_data_byte,
                pressure,
            }
        }
        // Control Change/Channel Mode
        0xB => {
            let value = read_byte_within(reader, within)?;
            ChannelMessageKind::ControlChange {
                control: first_data_byte,
                value,
            }
        }
        // Program Change
        0xC => ChannelMessageKind::ProgramChange(first_data_byte),
        // Channel Pressure
        0xD => ChannelMessageKind::ChannelPressure(first_data_byte),
        // Pitch Bend
        0xE => ChannelMessageKind::PitchBendChange(
            first_data_byte as u16 | ((read_byte_within(reader, within)? as u16) << 7),
        ),
        _ => return Err("System message present where channel message expected".into()),
    };
    Ok(ChannelMessage { channel, kind })
}

fn read_bytes<R: Read, const N: usize>(reader: &mut R) -> std::io::Result<[u8; N]> {
    let mut bytes = [0u8; N];
    reader.read_exact(&mut bytes)?;
    Ok(bytes)
}
fn read_u16<R: Read>(reader: &mut R) -> std::io::Result<u16> {
    let bytes = read_bytes(reader)?;
    Ok(u16::from_be_bytes(bytes))
}
fn read_u32<R: Read>(reader: &mut R) -> std::io::Result<u32> {
    let bytes = read_bytes(reader)?;
    Ok(u32::from_be_bytes(bytes))
}
fn read_byte_within<R: Read>(reader: &mut R, within: &mut u32) -> Result<u8, Box<dyn Error>> {
    if *within < 1 {
        return Err("Unterminated sequence within chunk".into());
    }
    let [byte] = read_bytes(reader)?;
    *within -= 1;
    Ok(byte)
}
fn read_variable_length_quantity_within<R: Read>(
    reader: &mut R,
    within: &mut u32,
) -> Result<u32, Box<dyn Error>> {
    let mut quantity: u32 = 0;
    loop {
        let byte = read_byte_within(reader, within)?;
        quantity <<= 7;
        quantity |= (byte & 0x7f) as u32;
        if byte & 0x80 == 0 {
            break;
        }
    }
    Ok(quantity)
}

fn write_midi(path: PathBuf, mut data: MidiData) -> Result<(), Box<dyn Error>> {
    // Order the data such that all time deltas are positive. For optimal space
    // use, order by channel secondarily also.
    data.channel_messages
        .sort_by_key(|&(time, ChannelMessage { channel, .. })| {
            ((time as u64) << 4) | (channel as u64)
        });
    data.other_events.sort_by_key(|&(time, _)| time);

    let mut file = BufWriter::new(File::create(path)?);

    eprintln!("Writing MIDI file (Standard MIDI File format 0).");

    // Write header chunk

    write_bytes(&mut file, b"MThd")?;
    write_u32(&mut file, 6)?;
    write_u16(&mut file, 0)?; // format 0
    write_u16(&mut file, 1)?; // one track
    write_u16(
        &mut file,
        match data.division {
            Division::TicksPerQuarterNote(ticks) => ticks,
            Division::TicksPerFrame {
                frame_rate,
                ticks_per_frame,
            } => (frame_rate as i8 as u16) << 8 | (ticks_per_frame as u16),
        },
    )?;

    // Write track chunk with events

    write_bytes(&mut file, b"MTrk")?;
    let length_pos = file.stream_position()?;
    write_u32(&mut file, 0)?; // placeholder length to be fixed up later

    let mut length = 0;
    let mut last_time: AbsoluteTime = 0;
    let mut running_status = None;

    let mut channel_messages = data.channel_messages.into_iter().peekable();
    let mut other_events = data.other_events.into_iter().peekable();
    loop {
        // Pick the iterator to advance such that no events will be out of order
        // in time, but SysEx messages and meta events precede channel messages.
        // This is an arbitrary ordering choice and probably not always correct,
        // but I think common metadata and SysEx messages like GM System Enable
        // make more sense if they precede any note data with the same timing?
        // It would be safer of course to not use two lists, but I like the
        // space-efficiency :(
        let process_other = match (other_events.peek(), channel_messages.peek()) {
            (Some((time_other, _)), Some((time_message, _))) => time_other <= time_message,
            (Some(_), None) => true,
            (None, Some(_)) => false,
            (None, None) => break,
        };

        if process_other {
            let (new_time, event_bytes) = other_events.next().unwrap();
            let delta_time = new_time - last_time;
            write_variable_length_quantity_within(&mut file, &mut length, delta_time)?;
            last_time = new_time;

            match event_bytes[0] {
                // SysEx start/continuation
                0xF0 | 0xF7 => {
                    write_byte_within(&mut file, &mut length, event_bytes[0])?;
                    running_status = None;
                    let sysex_bytes = &event_bytes[1..];
                    write_variable_length_quantity_within(
                        &mut file,
                        &mut length,
                        sysex_bytes.len().try_into().unwrap(),
                    )?;
                    for &sysex_byte in sysex_bytes {
                        write_byte_within(&mut file, &mut length, sysex_byte)?;
                    }
                }
                // Meta event
                0xFF => {
                    write_byte_within(&mut file, &mut length, event_bytes[0])?;
                    running_status = None;
                    write_byte_within(&mut file, &mut length, event_bytes[1])?;
                    let meta_bytes = &event_bytes[2..];
                    write_variable_length_quantity_within(
                        &mut file,
                        &mut length,
                        meta_bytes.len().try_into().unwrap(),
                    )?;
                    for &meta_byte in meta_bytes {
                        write_byte_within(&mut file, &mut length, meta_byte)?;
                    }
                }
                _ => unreachable!(),
            }
            continue;
        }

        let (new_time, message) = channel_messages.next().unwrap();
        let delta_time = new_time - last_time;
        write_variable_length_quantity_within(&mut file, &mut length, delta_time)?;
        last_time = new_time;

        let new_status = message.channel | (message.kind.discriminant() << 4);
        if running_status != Some(new_status) {
            running_status = Some(new_status);
            write_byte_within(&mut file, &mut length, new_status)?;
        }

        match message.kind {
            ChannelMessageKind::NoteOff {
                key: a,
                velocity: b,
            }
            | ChannelMessageKind::NoteOn {
                key: a,
                velocity: b,
            }
            | ChannelMessageKind::PolyKeyPressure {
                key: a,
                pressure: b,
            }
            | ChannelMessageKind::ControlChange {
                control: a,
                value: b,
            } => {
                write_byte_within(&mut file, &mut length, a)?;
                write_byte_within(&mut file, &mut length, b)?;
            }
            ChannelMessageKind::PitchBendChange(value) => {
                write_byte_within(&mut file, &mut length, (value & 0x7f) as u8)?;
                write_byte_within(&mut file, &mut length, (value >> 7) as u8)?;
            }
            ChannelMessageKind::ProgramChange(a) | ChannelMessageKind::ChannelPressure(a) => {
                write_byte_within(&mut file, &mut length, a)?;
            }
        }
    }

    // Fix up the length
    file.seek(SeekFrom::Start(length_pos))?;
    write_u32(&mut file, length)?;

    file.flush()?;

    eprintln!("Done writing MIDI file.");

    Ok(())
}

fn write_bytes<W: Write>(writer: &mut W, bytes: &[u8]) -> std::io::Result<()> {
    writer.write_all(bytes)
}
fn write_u16<W: Write>(writer: &mut W, value: u16) -> std::io::Result<()> {
    write_bytes(writer, &u16::to_be_bytes(value))
}
fn write_u32<W: Write>(writer: &mut W, value: u32) -> std::io::Result<()> {
    write_bytes(writer, &u32::to_be_bytes(value))
}
fn write_byte_within<W: Write>(
    writer: &mut W,
    within: &mut u32,
    byte: u8,
) -> Result<(), Box<dyn Error>> {
    if *within == u32::MAX {
        return Err("Chunk size overflow during writing".into());
    }
    *within += 1;
    write_bytes(writer, &[byte])?;
    Ok(())
}
fn write_variable_length_quantity_within<W: Write>(
    writer: &mut W,
    within: &mut u32,
    mut quantity: u32,
) -> Result<(), Box<dyn Error>> {
    let mut septet_count = if quantity < 1 << 7 {
        1
    } else if quantity < 1 << (7 * 2) {
        2
    } else if quantity < 1 << (7 * 3) {
        3
    } else if quantity < 1 << (7 * 4) {
        4
    } else {
        return Err("Variable-length quantity overflow during writing".into());
    };
    quantity <<= 32 - (7 * septet_count);

    loop {
        let septet = (quantity >> (32 - 7)) as u8;
        quantity <<= 7;
        septet_count -= 1;
        if septet_count == 0 {
            write_byte_within(writer, within, septet)?;
            break;
        } else {
            write_byte_within(writer, within, 0x80 | septet)?;
        }
    }
    Ok(())
}

const USAGE: &str = "\
unarpeggiator by hikari_no_yume

Usage:

    unarpeggiator arpeggio.mid [-o unarpegg.mid] [-v]

The input file is Standard MIDI File format 0 or format 1.

Options:

    -h
    --help
        Print this help text.

    -o <path>
        Writes MIDI in SMF format 0 to <path>.

    -v
        Verbose mode.
";

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = std::env::args_os();
    let _ = args.next(); // ignore argv[0]

    let mut in_path = None;
    let mut out_path = None;
    let mut verbose = false;
    while let Some(arg) = args.next() {
        if arg == "-v" {
            verbose = true;
        } else if arg == "-h" || arg == "--help" {
            eprintln!("{}", USAGE);
            return Ok(());
        } else if arg == "-o" {
            if out_path.is_some() {
                return Err("Only one output path can be specified".into());
            }
            out_path = args.next().map(PathBuf::from);
            if out_path.is_none() {
                return Err("Missing output path after -o".into());
            }
        } else if in_path.is_none() {
            in_path = Some(PathBuf::from(arg));
        } else {
            return Err(format!("Unexpected argument: {:?}", arg).into());
        }
    }

    let Some(in_path) = in_path else {
        eprintln!("{}", USAGE);
        return Err("No input path specified".into());
    };

    let data = read_midi(in_path, verbose)?;

    if let Some(out_path) = out_path {
        write_midi(out_path, data)?;
    } else {
        eprintln!("No output path specified, writing nothing.");
    }

    Ok(())
}
