use super::ast::ResolvedExpr;

/// An abstract musical event, completely independent of MIDI or specific synthesizers.
#[derive(Debug)]
pub struct SeqEvent {
    pub degree: i32,
    pub start_step: u32,
    pub duration_steps: u32,
}

#[derive(Debug, Default)]
pub struct TraversalContext {
    pub current_step: u32,
    pub sequence: Vec<SeqEvent>,
}

impl TraversalContext {
    pub fn walk_all(&mut self, expressions: &[ResolvedExpr]) {
        for expr in expressions {
            self.walk(expr);
        }
    }

    pub fn walk(&mut self, expr: &ResolvedExpr) {
        match expr {
            ResolvedExpr::Interval(n) => {
                // We record the abstract scale degree and timing, nothing more.
                self.sequence.push(SeqEvent {
                    degree: *n,
                    start_step: self.current_step,
                    duration_steps: 1, // Default to 1 abstract step
                });
                
                self.current_step += 1;
            }
            // Match against Pattern instead of List
            ResolvedExpr::Pattern(pattern) => {
                for child in pattern {
                    self.walk(child);
                }
            }
        }
    }
}
