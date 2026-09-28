use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Interval { index: i32, weight: u32 },
    Rest { weight: u32 },
    Pattern { elements: Vec<Expr>, weight: u32 },
    Chord { elements: Vec<Expr>, weight: u32 },
    Ident(String),
    Symbol(String),
    Str(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ResolvedExpr {
    Interval { index: i32, weight: u32 },
    Rest { weight: u32 },
    Pattern { elements: Vec<ResolvedExpr>, weight: u32 },
    Chord { elements: Vec<ResolvedExpr>, weight: u32 },
}

impl ResolvedExpr {
    /// Helper to get the weight of any resolved expression for time calculation
    pub fn weight(&self) -> u32 {
        match self {
            ResolvedExpr::Interval { weight, .. } => *weight,
            ResolvedExpr::Rest { weight } => *weight,
            ResolvedExpr::Pattern { weight, .. } => *weight,
            ResolvedExpr::Chord { weight, .. } => *weight,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Scene {
    pub configs: HashMap<String, Expr>,
    pub aliases: HashMap<String, Expr>,
    pub expressions: Vec<Expr>,
}

impl Scene {
    pub fn resolve_all(&self) -> Result<Vec<ResolvedExpr>, String> {
        self.expressions
            .iter()
            .map(|expr| self.resolve_expr(expr, 0))
            .collect()
    }

    fn resolve_expr(&self, expr: &Expr, depth: usize) -> Result<ResolvedExpr, String> {
        if depth > 32 {
            return Err("Max alias expansion depth exceeded".to_string());
        }

        match expr {
            Expr::Interval { index, weight } => Ok(ResolvedExpr::Interval { index: *index, weight: *weight }),
            Expr::Rest { weight } => Ok(ResolvedExpr::Rest { weight: *weight }),
            Expr::Pattern { elements, weight } => {
                let resolved = elements
                    .iter()
                    .map(|e| self.resolve_expr(e, depth))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(ResolvedExpr::Pattern { elements: resolved, weight: *weight })
            }
            Expr::Chord { elements, weight } => {
                let resolved = elements
                    .iter()
                    .map(|e| self.resolve_expr(e, depth))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(ResolvedExpr::Chord { elements: resolved, weight: *weight })
            }
            Expr::Ident(name) => {
                let alias_expr = self
                    .aliases
                    .get(name)
                    .ok_or_else(|| format!("Unresolved alias: '{}'", name))?;
                // Recursively resolve the alias
                self.resolve_expr(alias_expr, depth + 1)
            }
            Expr::Symbol(s) => Err(format!("Unexpected musical symbol in sequence: {}", s)),
            Expr::Str(s) => Err(format!("Unexpected string in sequence: {}", s)),
        }
    }
}
