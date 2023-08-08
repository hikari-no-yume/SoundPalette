use std::error::Error;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::PathBuf;

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

#[derive(Debug)]
struct Event {
    time: u32,
    kind: EventKind,
}

#[derive(Debug)]
enum EventKind {
    ChannelMessage(ChannelMessage),
    // TODO: SysEx and meta events
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

fn read_midi(path: PathBuf) -> Result<(), Box<dyn Error>> {
    let mut file = BufReader::new(File::open(path)?);

    eprintln!("Reading MIDI file.");

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
        0 => eprintln!("Standard MIDI File format 0"),
        // One song across several tracks
        1 => eprintln!("Standard MIDI File format 1"),
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
        eprintln!("Track {} ({} bytes):", trk, chunk_len);

        let mut bytes_left = chunk_len;

        // Read events

        let mut running_status = None;
        while bytes_left > 0 {
            let delta_time = read_variable_length_quantity_within(&mut file, &mut bytes_left)?;
            if delta_time > 0 {
                eprintln!("Delta time: +{} ticks", delta_time);
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
                    eprintln!("SysEx start ({} bytes)", length);
                    for _ in 0..length {
                        read_byte_within(&mut file, &mut bytes_left)?;
                    }
                }
                0xF7 => {
                    running_status = None;
                    let length = read_variable_length_quantity_within(&mut file, &mut bytes_left)?;
                    eprintln!("SysEx continuation ({} bytes)", length);
                    for _ in 0..length {
                        read_byte_within(&mut file, &mut bytes_left)?;
                    }
                }
                0xFF => {
                    running_status = None;
                    let type_ = read_byte_within(&mut file, &mut bytes_left)?;
                    if type_ >= 128 {
                        return Err("Invalid meta event type".into());
                    }
                    let length = read_variable_length_quantity_within(&mut file, &mut bytes_left)?;
                    eprintln!("Meta event type {:02X} ({} bytes)", type_, length);
                    for _ in 0..length {
                        read_byte_within(&mut file, &mut bytes_left)?;
                    }
                    if type_ == 0x2F {
                        eprintln!("End of track.");
                    }
                }
                _ => {
                    // This is a MIDI channel message. It may begin with a
                    // status byte to change the message kind and channel, or it
                    // may omit it (Running Status). The remaining bytes are
                    // always the data bytes, which depend on the kind.
                    let (status, first_data_byte) = if first_byte & 0x80 != 0 {
                        eprintln!(
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
                    eprintln!("{:?}", message);
                }
            }
        }
    }

    eprintln!("Reached final track, done reading MIDI file.");

    Ok(())
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

fn get_path() -> Result<PathBuf, &'static str> {
    let mut args = std::env::args_os();
    let _ = args.next(); // ignore argv[0]
    let path = args.next().map(PathBuf::from).ok_or("No path specified")?;
    match args.next() {
        Some(_) => Err("Too many arguments"),
        None => Ok(path),
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    read_midi(get_path()?)?;

    Ok(())
}
