use super::ast::ResolvedExpr;

#[derive(Debug)]
pub struct MidiNote {
    pub pitch: u8,
    pub velocity: u8,
    pub start_tick: u32,
    pub duration: u32,
}

#[derive(Debug, Default)]
pub struct TraversalContext {
    pub current_tick: u32,
    pub track: Vec<MidiNote>,
}

impl TraversalContext {
    pub fn walk_all(&mut self, expressions: &[ResolvedExpr]) {
        for expr in expressions {
            self.walk(expr);
        }
    }

    pub fn walk(&mut self, expr: &ResolvedExpr) {
        match expr {
            ResolvedExpr::Num(n) => {
                // Clamp the number to valid MIDI pitch ranges (0-127)
                let pitch = (*n).clamp(0, 127) as u8;
                
                self.track.push(MidiNote {
                    pitch,
                    velocity: 100, // Default velocity
                    start_tick: self.current_tick,
                    duration: 480, // Default quarter-note duration in ticks
                });
                
                // Advance the "playhead" by one quarter note
                self.current_tick += 480;
            }
            ResolvedExpr::List(list) => {
                // Treat a list as a "Sequence": play elements one after another.
                // The playhead advances naturally as we walk each child.
                for child in list {
                    self.walk(child);
                }
            }
        }
    }
}
