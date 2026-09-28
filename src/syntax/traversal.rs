use super::ast::ResolvedExpr;

/// An abstract musical event, completely independent of MIDI or specific synthesizers.
#[derive(Debug)]
pub struct SeqEvent {
    pub index: i32,
    pub start_step: f64,
    pub duration_steps: f64,
}

#[derive(Debug, Default)]
pub struct TraversalContext {
    pub current_step: f64,
    pub sequence: Vec<SeqEvent>,
}

impl TraversalContext {
    pub fn walk_all(&mut self, expressions: &[ResolvedExpr]) {
        for expr in expressions {
            // Top-level expressions occupy exactly 1 beat on the timeline grid.
            // The weight acts as a duration multiplier (e.g., 0@4 sustains for 4 beats).
            let actual_duration = 1.0 * (expr.weight() as f64);
            self.walk(expr, self.current_step, actual_duration);
            
            // Always advance the global grid by exactly 1 beat, regardless of sustain.
            self.current_step += 1.0;
        }
    }

    pub fn walk(&mut self, expr: &ResolvedExpr, start: f64, duration: f64) {
        match expr {
            ResolvedExpr::Interval { index, .. } => {
                // Leaf node: consume the allocated duration
                self.sequence.push(SeqEvent {
                    index: *index,
                    start_step: start,
                    duration_steps: duration,
                });
            }
            ResolvedExpr::Rest { .. } => {
                // Do nothing! The timeline advances naturally in the parent block.
            }
            ResolvedExpr::Pattern { elements, .. } => {
                let len = elements.len();
                if len == 0 { return; }
                
                // 1. The grid step size is strictly divided by the number of elements
                let step_size = duration / (len as f64);
                
                // 2. Portion out the time
                let mut current_start = start;
                for child in elements {
                    // The child note's sustain duration is multiplied by its weight
                    let child_duration = step_size * (child.weight() as f64);
                    
                    self.walk(child, current_start, child_duration);
                    
                    // The sequence grid ALWAYS advances by exactly 1 step size, 
                    // ensuring subsequent notes aren't pushed out of time!
                    current_start += step_size; 
                }
            }
            ResolvedExpr::Chord { elements, .. } => {
                // Parallel: Every child gets the EXACT SAME start time.
                // Their individual sustain is modified by their weight.
                for child in elements {
                    let child_duration = duration * (child.weight() as f64);
                    self.walk(child, start, child_duration);
                }
            }
        }
    }
}
