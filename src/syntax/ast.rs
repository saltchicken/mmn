use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Interval {
        index: i32,
        velocity: Option<u32>,
        weight: u32,
    },
    Rest {
        weight: u32,
    },
    Pattern {
        elements: Vec<Expr>,
        velocity: Option<u32>,
        weight: u32,
    },
    Chord {
        elements: Vec<Expr>,
        velocity: Option<u32>,
        weight: u32,
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
        weight: u32,
    },
    Rest {
        weight: u32,
    },
    Pattern {
        elements: Vec<ResolvedExpr>,
        weight: u32,
    },
    Chord {
        elements: Vec<ResolvedExpr>,
        weight: u32,
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

        match expr {
            Expr::Interval {
                index,
                velocity,
                weight,
            } => Ok(ResolvedExpr::Interval {
                index: *index,
                velocity: velocity.unwrap_or(inherited_vel), // Inherit if None!
                weight: *weight,
            }),
            Expr::Rest { weight } => Ok(ResolvedExpr::Rest { weight: *weight }),

            Expr::Pattern {
                elements,
                velocity,
                weight,
            } => {
                // Determine what velocity we pass to our children
                let current_vel = velocity.unwrap_or(inherited_vel);
                let resolved = elements
                    .iter()
                    .map(|e| self.resolve_expr(e, depth, current_vel))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(ResolvedExpr::Pattern {
                    elements: resolved,
                    weight: *weight,
                })
            }

            Expr::Chord {
                elements,
                velocity,
                weight,
            } => {
                let current_vel = velocity.unwrap_or(inherited_vel);
                let resolved = elements
                    .iter()
                    .map(|e| self.resolve_expr(e, depth, current_vel))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(ResolvedExpr::Chord {
                    elements: resolved,
                    weight: *weight,
                })
            }

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
