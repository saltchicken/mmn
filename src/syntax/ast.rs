use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Interval { index: i32, velocity: u32, weight: u32 }, // Added velocity
    Rest { weight: u32 },
    Pattern { elements: Vec<Expr>, weight: u32 },
    Chord { elements: Vec<Expr>, weight: u32 },
    Ident(String),
    Symbol(String),
    Str(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ResolvedExpr {
    Interval { index: i32, velocity: u32, weight: u32 }, // Added velocity
    Rest { weight: u32 },
    Pattern { elements: Vec<ResolvedExpr>, weight: u32 },
    Chord { elements: Vec<ResolvedExpr>, weight: u32 },
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
            .map(|expr| self.resolve_expr(expr, 0))
            .collect()
    }

    fn resolve_expr(&self, expr: &Expr, depth: usize) -> Result<ResolvedExpr, String> {
        if depth > 32 {
            return Err("Max alias expansion depth exceeded".to_string());
        }

        match expr {
            // Pass velocity through the resolution step
            Expr::Interval { index, velocity, weight } => Ok(ResolvedExpr::Interval { 
                index: *index, 
                velocity: *velocity, 
                weight: *weight 
            }),
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
                let alias_expr = self.aliases.get(name)
                    .ok_or_else(|| format!("Unresolved alias: '{}'", name))?;
                self.resolve_expr(alias_expr, depth + 1)
            }
            
            Expr::Symbol(s) => Err(format!("Unexpected musical symbol in sequence: {}", s)),
            Expr::Str(s) => Err(format!("Unexpected string in sequence: {}", s)),
        }
    }
}
