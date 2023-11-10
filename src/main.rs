// This crate will be called SoundPalette whether Rust likes it or not.
#![allow(non_snake_case)]

use libSoundPalette::midi::{read_midi, write_midi};

use std::error::Error;
use std::path::PathBuf;

const USAGE: &str = "\
SoundPalette CLI tool by hikari_no_yume

Usage:

    SoundPalette arpeggio.mid [-o unarpegg.mid] [-s] [-v]

The input file is Standard MIDI File format 0 or format 1.

Options:

    -h
    --help
        Print this help text.

    -o <path>
        Writes MIDI in SMF format 0 to <path>. The output MIDI file
        should be more or less equivalent to the input MIDI file: the
        timing and content of MIDI events will not be changed, but
        formatting details and ordering may differ.

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

    let data = read_midi(in_path, verbose)?;

    if let Some(out_path) = out_path {
        write_midi(out_path, data)?;
    } else {
        eprintln!("No output path specified, writing nothing.");
    }

    Ok(())
}
