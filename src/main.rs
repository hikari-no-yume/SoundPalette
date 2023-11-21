/*
 * Part of SoundPalette by hikari_no_yume.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
// This crate will be called SoundPalette whether Rust likes it or not.
#![allow(non_snake_case)]

use libSoundPalette::midi::{format_bytes, read_midi, write_midi};
use libSoundPalette::sysex::{generate_sysex, SysExGenerator};
use libSoundPalette::ui::{list_other_events, print_menu, StderrTableStream};

use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};
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

    --list-sysex-generators
        List all types of SysEx that can be generated.
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
        } else if arg == "--list-sysex-generators" {
            print_menu(&generate_sysex(), &|generator: Box<dyn SysExGenerator>| {
                let mut sysex_bytes = Vec::new();
                generator.generate(&mut sysex_bytes);
                eprint!("{}", format_bytes(&sysex_bytes));
            });
            return Ok(());
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

    let mut data = read_midi(
        &mut BufReader::new(File::open(in_path)?),
        verbose,
        &mut std::io::stderr(),
    )?;

    list_other_events(
        &mut StderrTableStream::new(),
        &data,
        /* with_time_and_kind: */ true,
    );

    if let Some(out_path) = out_path {
        let mut file = BufWriter::new(File::create(out_path)?);
        write_midi(&mut file, &mut data, &mut std::io::stderr())?;
    } else {
        eprintln!("No output path specified, writing nothing.");
    }

    Ok(())
}
