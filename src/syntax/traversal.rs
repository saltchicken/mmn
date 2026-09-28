use super::ast::{Expr, ResolvedExpr};
use std::collections::HashMap;

#[derive(Debug)]
pub struct MidiNote {
    pub pitch: u8,
    pub velocity: u8,
    pub start_tick: u32,
    pub duration: u32,
}

#[derive(Debug)]
pub struct TraversalContext {
    pub current_tick: u32,
    pub track: Vec<MidiNote>,

    pub root_note: u8,
    pub scale: Vec<u8>,
    pub bpm: u32,
}

impl Default for TraversalContext {
    fn default() -> Self {
        Self {
            current_tick: 0,
            track: Vec::new(),
            root_note: 60,
            scale: vec![0, 2, 4, 5, 7, 9, 11],
            bpm: 120,
        }
    }
}

fn match_pitch_class(note: &str) -> Option<i32> {
    match note.to_uppercase().as_str() {
        "C" => Some(0),
        "C#" | "DB" => Some(1),
        "D" => Some(2),
        "D#" | "EB" => Some(3),
        "E" => Some(4),
        "F" => Some(5),
        "F#" | "GB" => Some(6),
        "G" => Some(7),
        "G#" | "AB" => Some(8),
        "A" => Some(9),
        "A#" | "BB" => Some(10),
        "B" => Some(11),
        _ => None,
    }
}

impl TraversalContext {
    /// Applies scene configs and returns an Err if required configs are missing or invalid
    pub fn apply_scene_configs(&mut self, configs: &HashMap<String, Expr>) -> Result<(), String> {
        // 1. Enforce required configs
        let required_keys = ["ROOT", "SCALE", "BPM"];
        for key in required_keys {
            if !configs.contains_key(key) {
                return Err(format!("Missing required configuration: #{}", key));
            }
        }

        let mut root_class = 0;
        let mut octave = 4; // Default to octave 4 if #OCTAVE isn't specified

        // 2. Process all configs
        for (name, expr) in configs {
            match name.as_str() {
                "ROOT" => {
                    if let Expr::Ref(s) = expr {
                        if let Some(c) = match_pitch_class(s) {
                            root_class = c;
                        } else {
                            return Err(format!("Invalid ROOT pitch class '{}'", s));
                        }
                    } else {
                        return Err("#ROOT must be a note like C or C#".to_string());
                    }
                }
                "OCTAVE" => {
                    if let Expr::Num(n) = expr {
                        octave = *n;
                    } else {
                        return Err("#OCTAVE must be a number".to_string());
                    }
                }
                "SCALE" => {
                    if let Expr::Ref(s) = expr {
                        self.apply_scale(s)?;
                    } else {
                        return Err("#SCALE must be a string like minor or major".to_string());
                    }
                }
                "BPM" => {
                    if let Expr::Num(n) = expr {
                        if *n > 0 {
                            self.bpm = *n as u32;
                        } else {
                            return Err("#BPM must be a positive number".to_string());
                        }
                    } else {
                        return Err("#BPM must be a number".to_string());
                    }
                }
                _ => {
                    println!("Warning: Unknown config directive '#{}'", name);
                }
            }
        }

        // 3. Finalize MIDI root note
        let midi_note = (octave + 1) * 12 + root_class;

        if (0..=127).contains(&midi_note) {
            self.root_note = midi_note as u8;
        } else {
            return Err(format!(
                "Calculated root note {} is out of MIDI range (0-127). Check your #ROOT and #OCTAVE values.",
                midi_note
            ));
        }

        Ok(())
    }

    fn apply_scale(&mut self, scale_name: &str) -> Result<(), String> {
        self.scale = match scale_name.to_lowercase().as_str() {
            "minor" | "aeolian" => vec![0, 2, 3, 5, 7, 8, 10],
            "major" | "ionian" => vec![0, 2, 4, 5, 7, 9, 11],
            "dorian" => vec![0, 2, 3, 5, 7, 9, 10],
            "phrygian" => vec![0, 1, 3, 5, 7, 8, 10],
            "lydian" => vec![0, 2, 4, 6, 7, 9, 11],
            "mixolydian" => vec![0, 2, 4, 5, 7, 9, 10],
            "locrian" => vec![0, 1, 3, 5, 6, 8, 10],
            "pentatonic" => vec![0, 2, 4, 7, 9],
            "minor_pentatonic" => vec![0, 3, 5, 7, 10],
            _ => return Err(format!("Unknown scale '{}'", scale_name)),
        };
        Ok(())
    }

    pub fn walk_all(&mut self, expressions: &[ResolvedExpr]) {
        for expr in expressions {
            self.walk(expr);
        }
    }

    fn calculate_pitch(&self, degree: i32) -> u8 {
        let scale_len = self.scale.len() as i32;
        let octave_shift = degree.div_euclid(scale_len);
        let scale_index = degree.rem_euclid(scale_len) as usize;

        let pitch_offset = (octave_shift * 12) + self.scale[scale_index] as i32;
        let final_pitch = (self.root_note as i32) + pitch_offset;

        final_pitch.clamp(0, 127) as u8
    }

    pub fn walk(&mut self, expr: &ResolvedExpr) {
        match expr {
            ResolvedExpr::Num(n) => {
                let pitch = self.calculate_pitch(*n);

                self.track.push(MidiNote {
                    pitch,
                    velocity: 100,
                    start_tick: self.current_tick,
                    duration: 480,
                });

                self.current_tick += 480;
            }
            ResolvedExpr::List(list) => {
                for child in list {
                    self.walk(child);
                }
            }
        }
    }
}
