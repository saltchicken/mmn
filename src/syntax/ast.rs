use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Interval {
        index: i32,
        velocity: Option<u32>,
        hold: f32,
    },
    Rest {
        hold: f32,
    },
    Pattern {
        elements: Vec<Expr>,
        velocity: Option<u32>,
        span: f32,
    },
    Chord {
        elements: Vec<Expr>,
        velocity: Option<u32>,
        hold: f32,
    },
    Ident(String),
    Symbol(String),
    Str(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ResolvedExpr {
    Interval {
        index: i32,
        velocity: u32,
        hold: f32,
    },
    Rest {
        hold: f32,
    },
    Pattern {
        elements: Vec<ResolvedExpr>,
        span: f32,
    },
    Chord {
        elements: Vec<ResolvedExpr>,
        hold: f32,
    },
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Scene {
    pub configs: HashMap<String, Expr>,
    pub aliases: HashMap<String, Expr>,
    pub expressions: Vec<Expr>,
}

impl Scene {
    pub fn resolve_all(&self) -> Result<Vec<ResolvedExpr>, String> {
        self.expressions
            .iter()
            // 127 is our global top-level default velocity
            .map(|expr| self.resolve_expr(expr, 0, 127))
            .collect()
    }

    // Notice we pass `inherited_vel` down through the recursive calls
    fn resolve_expr(
        &self,
        expr: &Expr,
        depth: usize,
        inherited_vel: u32,
    ) -> Result<ResolvedExpr, String> {
        if depth > 32 {
            return Err("Max alias expansion depth exceeded".to_string());
        }

        // Shared closure to handle mapping over nested pattern/chord elements
        let resolve_group = |elements: &[Expr], vel: Option<u32>| -> Result<Vec<ResolvedExpr>, String> {
            let current_vel = vel.unwrap_or(inherited_vel);
            elements
                .iter()
                .map(|e| self.resolve_expr(e, depth, current_vel))
                .collect()
        };

        match expr {
            Expr::Interval {
                index,
                velocity,
                hold,
            } => Ok(ResolvedExpr::Interval {
                index: *index,
                velocity: velocity.unwrap_or(inherited_vel), // Inherit if None!
                hold: *hold,
            }),
            Expr::Rest { hold } => Ok(ResolvedExpr::Rest { hold: *hold }),

            Expr::Pattern {
                elements,
                velocity,
                span,
            } => Ok(ResolvedExpr::Pattern {
                elements: resolve_group(elements, *velocity)?,
                span: *span,
            }),

            Expr::Chord {
                elements,
                velocity,
                hold,
            } => Ok(ResolvedExpr::Chord {
                elements: resolve_group(elements, *velocity)?,
                hold: *hold,
            }),

            Expr::Ident(name) => {
                let alias_expr = self
                    .aliases
                    .get(name)
                    .ok_or_else(|| format!("Unresolved alias: '{}'", name))?;

                // Aliases inherit the velocity of wherever they were called from
                self.resolve_expr(alias_expr, depth + 1, inherited_vel)
            }

            Expr::Symbol(s) => Err(format!("Unexpected musical symbol in sequence: {}", s)),
            Expr::Str(s) => Err(format!("Unexpected string in sequence: {}", s)),
        }
    }
}
