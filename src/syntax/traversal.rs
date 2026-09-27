use super::ast::ResolvedExpr;

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
    
    // Musical Context
    pub root_note: u8,   // 60 = Middle C
    pub scale: Vec<u8>,  // Semitone offsets from the root
}

impl Default for TraversalContext {
    fn default() -> Self {
        Self {
            current_tick: 0,
            track: Vec::new(),
            root_note: 60, // C4
            // Default to a Major Scale (Ionian)
            scale: vec![0, 2, 4, 5, 7, 9, 11],
        }
    }
}

impl TraversalContext {
    /// Applies scene configurations, overriding defaults
    pub fn apply_config(&mut self, root_note: Option<u8>, scale_name: Option<String>) {
        if let Some(r) = root_note {
            self.root_note = r;
        }
        
        if let Some(s) = scale_name {
            self.scale = match s.to_lowercase().as_str() {
                "minor" | "aeolian" => vec![0, 2, 3, 5, 7, 8, 10],
                "major" | "ionian" => vec![0, 2, 4, 5, 7, 9, 11],
                "dorian" => vec![0, 2, 3, 5, 7, 9, 10],
                "phrygian" => vec![0, 1, 3, 5, 7, 8, 10],
                "lydian" => vec![0, 2, 4, 6, 7, 9, 11],
                "mixolydian" => vec![0, 2, 4, 5, 7, 9, 10],
                "locrian" => vec![0, 1, 3, 5, 6, 8, 10],
                "pentatonic" => vec![0, 2, 4, 7, 9],
                "minor_pentatonic" => vec![0, 3, 5, 7, 10],
                _ => {
                    println!("Warning: Unknown scale '{}', defaulting to major", s);
                    vec![0, 2, 4, 5, 7, 9, 11]
                }
            };
        }
    }

    pub fn walk_all(&mut self, expressions: &[ResolvedExpr]) {
        for expr in expressions {
            self.walk(expr);
        }
    }

    /// Converts a scale degree into a MIDI pitch.
    /// 0 = Root, 1 = 2nd, 7 = Root (octave up), -1 = 7th (octave down)
    fn calculate_pitch(&self, degree: i32) -> u8 {
        let scale_len = self.scale.len() as i32;
        
        // div_euclid and rem_euclid correctly handle negative degrees
        let octave_shift = degree.div_euclid(scale_len);
        let scale_index = degree.rem_euclid(scale_len) as usize;
        
        let pitch_offset = (octave_shift * 12) + self.scale[scale_index] as i32;
        let final_pitch = (self.root_note as i32) + pitch_offset;
        
        // Ensure we don't crash DAW synths by going out of MIDI bounds
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
