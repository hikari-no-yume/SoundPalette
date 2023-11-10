// This crate will be called SoundPalette whether Rust likes it or not.
#![allow(non_snake_case)]

use libSoundPalette::midi::{
    read_midi, write_midi, AbsoluteTime, ChannelMessage, ChannelMessageKind,
};

use std::error::Error;
use std::path::PathBuf;

macro_rules! logif {
    ($verbose:ident, $($arg:tt)+) => {
        if $verbose {
            eprintln!($($arg)*);
        }
    }
}

fn unarpeggiate(messages: &mut [(AbsoluteTime, ChannelMessage)], v: bool) {
    // Sort by channel first and time second. This makes it easier and more
    // efficient to pair Note On and Note Off messages within a single channel.
    messages.sort_by_key(|&(time, ChannelMessage { channel, .. })| {
        ((channel as u64) << 32) | (time as u64)
    });

    eprintln!("Unarpeggiating.");

    fn find_note_on(
        messages: &[(AbsoluteTime, ChannelMessage)],
        start_idx: usize,
        channel: Option<u8>,
    ) -> Option<usize> {
        for (i, (_time, message)) in messages[start_idx..].iter().enumerate() {
            if let Some(channel) = channel {
                if message.channel != channel {
                    return None;
                }
            }
            match message.kind {
                // velocity = 0 in Note On is equivalent to Note Off
                ChannelMessageKind::NoteOn { velocity: 0, .. } => (),
                ChannelMessageKind::NoteOn { velocity: _, .. } => return Some(start_idx + i),
                _ => (),
            };
        }
        None
    }

    fn find_note_off(
        messages: &[(AbsoluteTime, ChannelMessage)],
        start_idx: usize,
        channel: u8,
        note: u8,
    ) -> Option<usize> {
        for (i, (_time, message)) in messages[start_idx..].iter().enumerate() {
            if message.channel != channel {
                return None;
            }
            match message.kind {
                // velocity = 0 in Note On is equivalent to Note Off
                ChannelMessageKind::NoteOff { key, .. }
                | ChannelMessageKind::NoteOn { key, velocity: 0 }
                    if key == note =>
                {
                    return Some(start_idx + i);
                }
                _ => (),
            };
        }
        None
    }

    let mut last_channel = None;
    // reusable vecs for the indices of the Note On and Note Off commands in
    // the current arpeggio
    //let mut current_arpeggio_on = Vec::new();
    //let mut current_arpeggio_off = Vec::new();

    let mut i = 0;
    loop {
        // Start by finding a Note On / Note Off pair.

        let Some(note_on_idx) = find_note_on(messages, i, None) else {
            // No more Note On messages.
            break;
        };

        let (first_time, note_on_message) = &messages[note_on_idx];
        let channel = note_on_message.channel;
        let ChannelMessageKind::NoteOn {
            key: first_note, ..
        } = note_on_message.kind
        else {
            unreachable!();
        };

        if last_channel != Some(channel) {
            logif!(v, "Channel {}:", channel);
            last_channel = Some(channel);
        }

        let Some(note_off_idx) = find_note_off(messages, note_on_idx + 1, channel, first_note)
        else {
            i = note_on_idx + 1;
            continue;
        };

        eprintln!(
            "Initial pair: {:?} {:?}",
            messages[note_on_idx], messages[note_off_idx]
        );

        // Try to match a subsequent Note On / Note Off pair.

        // The next Note On might be before the current Note Off.
        let mut j = note_on_idx + 1;
        #[allow(clippy::never_loop)]
        loop {
            let Some(note_on_idx2) = find_note_on(messages, j, Some(channel)) else {
                break;
            };
            let (new_time, note_on_message) = &messages[note_on_idx2];
            let ChannelMessageKind::NoteOn { key: new_note, .. } = note_on_message.kind else {
                unreachable!();
            };

            // If there's no time difference, this isn't an arpeggio.
            if new_time == first_time {
                break;
            }

            // Match major or minor third.
            // TODO: inversions, fifths, etc.
            if ![3, 4].contains(&new_note.abs_diff(first_note)) {
                break;
            }

            let Some(note_off_idx2) = find_note_off(messages, note_on_idx2 + 1, channel, new_note)
            else {
                break;
            };

            eprintln!(
                "Major/minor third pair to unarpeggiate: {:?} {:?}",
                messages[note_on_idx2], messages[note_off_idx2]
            );

            let min_time = messages[note_on_idx].0.min(messages[note_on_idx2].0);
            let max_time = messages[note_off_idx].0.max(messages[note_off_idx2].0);
            messages[note_on_idx].0 = min_time;
            messages[note_on_idx2].0 = min_time;
            messages[note_off_idx].0 = max_time;
            messages[note_off_idx2].0 = max_time;

            // TODO: match more pairs?
            break;
        }

        // TODO: unarpeggiate.

        i = note_on_idx + 1;
    }

    eprintln!("Done unarpeggiating.");
}

const USAGE: &str = "\
unarpeggiator by hikari_no_yume

Usage:

    unarpeggiator arpeggio.mid [-o unarpegg.mid] [-s] [-v]

The input file is Standard MIDI File format 0 or format 1.

Options:

    -h
    --help
        Print this help text.

    -o <path>
        Writes MIDI in SMF format 0 to <path>.

    -s
        Skip unarpeggiation. In this mode, the output MIDI file should
        be more or less equivalent to the input MIDI file: the timing
        and content of MIDI events will not be changed, but formatting
        details and ordering may differ.

    -v
        Verbose mode.
";

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = std::env::args_os();
    let _ = args.next(); // ignore argv[0]

    let mut in_path = None;
    let mut out_path = None;
    let mut skip_unarpeggiation = false;
    let mut verbose = false;
    while let Some(arg) = args.next() {
        if arg == "-h" || arg == "--help" {
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
        } else if arg == "-s" {
            skip_unarpeggiation = true;
        } else if arg == "-v" {
            verbose = true;
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

    let mut data = read_midi(in_path, verbose)?;

    if skip_unarpeggiation {
        eprintln!("Skipping unarpeggiation.");
    } else {
        unarpeggiate(&mut data.channel_messages, verbose);
    }

    if let Some(out_path) = out_path {
        write_midi(out_path, data)?;
    } else {
        eprintln!("No output path specified, writing nothing.");
    }

    Ok(())
}
