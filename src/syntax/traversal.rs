use super::ast::ResolvedExpr;

/// An abstract musical event, completely independent of MIDI or specific synthesizers.
#[derive(Debug)]
pub struct SeqEvent {
    pub degree: i32,
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
            // Each top-level expression takes up exactly 1 beat/step
            self.walk(expr, self.current_step, 1.0);
            self.current_step += 1.0;
        }
    }

    pub fn walk(&mut self, expr: &ResolvedExpr, start: f64, duration: f64) {
        match expr {
            ResolvedExpr::Interval(n) => {
                // Leaf node: consume the allocated duration
                self.sequence.push(SeqEvent {
                    degree: *n,
                    start_step: start,
                    duration_steps: duration,
                });
            }
            ResolvedExpr::Rest => {
                // Do nothing! The time is still accounted for because
                // the parent pattern allocated `start` and `duration`
                // for the *next* sibling automatically.
            }
            ResolvedExpr::Tie => {
                // Extend the duration of the last played note
                if let Some(last_event) = self.sequence.last_mut() {
                    last_event.duration_steps += duration;
                }
            }
            // Match against Pattern instead of List
            ResolvedExpr::Pattern(pattern) => {
                let len = pattern.len();
                if len == 0 {
                    return; // Ignore empty patterns
                }
                
                // Subdivide the available duration equally among the children
                let step_size = duration / (len as f64);
                
                for (i, child) in pattern.iter().enumerate() {
                    let child_start = start + (i as f64 * step_size);
                    self.walk(child, child_start, step_size);
                }
            }
        }
    }
}
