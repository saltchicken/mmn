// src/midi.rs
use std::collections::HashMap;
use crate::syntax::ast::Expr;
use crate::syntax::traversal::SeqEvent;

#[derive(Debug)]
pub struct MidiNote {
    pub pitch: u8,
    pub velocity: u8,
    pub start_tick: u32,
    pub duration: u32,
}

#[derive(Debug)]
pub struct MidiTrack {
    pub root_note: u8,
    pub scale: Vec<u8>,
    pub bpm: u32,
    pub notes: Vec<MidiNote>,
}

fn match_pitch_class(note: &str) -> Option<i32> {
    match note.to_uppercase().as_str() {
        "C" => Some(0), "C#" | "DB" => Some(1), "D" => Some(2),
        "D#" | "EB" => Some(3), "E" => Some(4), "F" => Some(5),
        "F#" | "GB" => Some(6), "G" => Some(7), "G#" | "AB" => Some(8),
        "A" => Some(9), "A#" | "BB" => Some(10), "B" => Some(11),
        _ => None,
    }
}

fn get_scale_intervals(scale_name: &str) -> Result<Vec<u8>, String> {
    match scale_name.to_lowercase().as_str() {
        "minor" | "aeolian" => Ok(vec![0, 2, 3, 5, 7, 8, 10]),
        "major" | "ionian" => Ok(vec![0, 2, 4, 5, 7, 9, 11]),
        "dorian" => Ok(vec![0, 2, 3, 5, 7, 9, 10]),
        "phrygian" => Ok(vec![0, 1, 3, 5, 7, 8, 10]),
        "lydian" => Ok(vec![0, 2, 4, 6, 7, 9, 11]),
        "mixolydian" => Ok(vec![0, 2, 4, 5, 7, 9, 10]),
        "locrian" => Ok(vec![0, 1, 3, 5, 6, 8, 10]),
        "pentatonic" => Ok(vec![0, 2, 4, 7, 9]),
        "minor_pentatonic" => Ok(vec![0, 3, 5, 7, 10]),
        _ => Err(format!("Unknown scale '{}'", scale_name)),
    }
}

/// Generates a concrete MIDI track from abstract sequences and configs
pub fn generate_midi(configs: &HashMap<String, Expr>, sequence: &[SeqEvent]) -> Result<MidiTrack, String> {
    let required_keys = ["SCALE", "BPM"];
    for key in required_keys {
        if !configs.contains_key(key) {
            return Err(format!("Missing required configuration: #{}", key));
        }
    }

    let mut root_class = 0;
    let mut octave = 4;
    let mut scale = vec![];
    let mut bpm = 120;

    for (name, expr) in configs {
        let val_str = match expr {
            Expr::Str(s) => s.as_str(),
            _ => return Err(format!("#{name} config must evaluate to a string")),
        };

        match name.as_str() {
            "SCALE" => {
                let parts: Vec<&str> = val_str.split_whitespace().collect();
                if parts.len() != 2 {
                    return Err("#SCALE must be in format '<NOTE><OCTAVE> <SCALE_NAME>', e.g. 'C4 minor'".to_string());
                }

                let note_str = parts[0].to_uppercase();
                let scale_name = parts[1];

                // Split at the first number found (to separate "C" or "C#" from "4")
                let split_idx = note_str.find(|c: char| c.is_ascii_digit())
                    .ok_or_else(|| format!("Missing octave in scale root: {}", parts[0]))?;
                
                let (note_part, octave_part) = note_str.split_at(split_idx);
                
                root_class = match_pitch_class(note_part)
                    .ok_or_else(|| format!("Invalid root note: {}", note_part))?;
                
                octave = octave_part.parse::<i32>()
                    .map_err(|_| format!("Invalid octave: {}", octave_part))?;

                scale = get_scale_intervals(scale_name)?;
            }
            "BPM" => {
                bpm = val_str.parse::<u32>().map_err(|_| "#BPM must be a valid positive integer".to_string())?;
            }
            _ => {}
        }
    }

    let midi_root_note = (octave + 1) * 12 + root_class;
    if !(0..=127).contains(&midi_root_note) {
        return Err(format!("Root note {} out of bounds (0-127)", midi_root_note));
    }

    let mut notes = Vec::new();
    let ticks_per_step = 480.0; // Standard MIDI resolution (Pulses Per Quarter Note) as f64

    for event in sequence {
        let scale_len = scale.len() as i32;
        let octave_shift = event.index.div_euclid(scale_len);
        let scale_index = event.index.rem_euclid(scale_len) as usize;
        
        let pitch_offset = (octave_shift * 12) + scale[scale_index] as i32;
        let final_pitch = midi_root_note + pitch_offset;

        notes.push(MidiNote {
            pitch: final_pitch.clamp(0, 127) as u8,
            velocity: 100,
            // Multiply float timing by PPQN and round to nearest integer tick
            start_tick: (event.start_step * ticks_per_step).round() as u32,
            duration: (event.duration_steps * ticks_per_step).round() as u32,
        });
    }

    Ok(MidiTrack {
        root_note: midi_root_note as u8,
        scale,
        bpm,
        notes,
    })
}
