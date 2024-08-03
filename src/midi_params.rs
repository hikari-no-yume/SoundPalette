/*
 * Part of SoundPalette by hikari_no_yume.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! Descriptions of MIDI parameter values that are not closed classes, like note
//! (drum), controller and program numbers.

use crate::ui::{FlexibleDisplay, FlexibleFormatter};
use std::fmt::Result as FmtResult;

pub struct NoteNumberDescriber {
    pub note_number: u8,
    pub show_drum: bool,
}
impl FlexibleDisplay for NoteNumberDescriber {
    fn fmt_flexible(&self, f: &mut impl FlexibleFormatter) -> FmtResult {
        let note = self.note_number;
        assert!((0x00..=0x7F).contains(&note));

        let pitch_class = [
            "C", "C♯", "D", "D♯", "E", "F", "F♯", "G", "G♯", "A", "A♯", "B",
        ][usize::from(note % 12)];
        let octave = (note as i16) / 12 - 1;
        write!(f, "{}{}", pitch_class, octave)?;

        if self.show_drum {
            let gm1_drum = note
                .checked_sub(GM1_DRUMS_BASE)
                .and_then(|drum| GM1_DRUMS.get(usize::from(drum)));
            let gs_drum1 = note
                .checked_sub(GS_EXTRA_DRUMS_1_BASE)
                .and_then(|drum| GS_EXTRA_DRUMS_1.get(usize::from(drum)));
            let gs_drum2 = note
                .checked_sub(GS_EXTRA_DRUMS_2_BASE)
                .and_then(|drum| GS_EXTRA_DRUMS_2.get(usize::from(drum)));
            if gm1_drum.is_some() || gs_drum1.is_some() || gs_drum2.is_some() {
                f.punctuate(" [")?;
                f.begin_span("param-value-name")?;
                if let Some(gm1_drum) = gm1_drum {
                    write!(f, "GM1: {}", gm1_drum)?;
                }
                if let Some(gs_drum) = gs_drum1.or(gs_drum2) {
                    write!(f, "GS: {}", gs_drum)?;
                }
                f.end_span()?;
                f.punctuate("]")?;
            }
        }

        Ok(())
    }
}

pub const GS_EXTRA_DRUMS_1: &[&str] = &[
    "High Q",
    "Slap",
    "Scratch Push",
    "Scratch Pull",
    "Sticks",
    "Square Click",
    "Metronome Click",
    "Metronome Bell",
];
pub const GS_EXTRA_DRUMS_1_BASE: u8 = 27;

pub const GM1_DRUMS: &[&str] = &[
    "Acoustic Bass Drum",
    "Bass Drum 1",
    "Side Stick",
    "Acoustic Snare",
    "Hand Clap",
    "Electric Snare",
    "Low Floor Tom",
    "Closed Hi Hat",
    "High Floor Tom",
    "Pedal Hi-Hat",
    "Low Tom",
    "Open Hi-Hat",
    "Low-Mid Tom",
    "Hi Mid Tom",
    "Crash Cymbal 1",
    "High Tom",
    "Ride Cymbal 1",
    "Chinese Cymbal",
    "Ride Bell",
    "Tambourine",
    "Splash Cymbal",
    "Cowbell",
    "Crash Cymbal 2",
    "Vibraslap",
    "Ride Cymbal 2",
    "Hi Bongo",
    "Low Bongo",
    "Mute Hi Conga",
    "Open Hi Conga",
    "Low Conga",
    "High Timbale",
    "Low Timbale",
    "High Agogo",
    "Low Agogo",
    "Cabasa",
    "Maracas",
    "Short Whistle",
    "Long Whistle",
    "Short Guiro",
    "Long Guiro",
    "Claves",
    "Hi Wood Block",
    "Low Wood Block",
    "Mute Cuica",
    "Open Cuica",
    "Mute Triangle",
    "Open Triangle",
];
pub const GM1_DRUMS_BASE: u8 = 35;

pub const GS_EXTRA_DRUMS_2: &[&str] = &[
    "Shaker",
    "Jingle Bell",
    "Bell Tree",
    "Castanets",
    "Mute Surdo",
    "Open Surdo",
    "Applause [Orchestra Set]",
];
pub const GS_EXTRA_DRUMS_2_BASE: u8 = 82;
