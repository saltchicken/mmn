use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Num(i32),
    List(Vec<Expr>),
    Ident(String),
    Symbol(String),
    Str(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ResolvedExpr {
    Interval(i32),
    List(Vec<ResolvedExpr>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Scene {
    // Configs are now just stored as raw parsed key-value expressions
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
            Expr::Num(n) => Ok(ResolvedExpr::Interval(*n)),
            Expr::List(list) => {
                let resolved_list = list
                    .iter()
                    .map(|e| self.resolve_expr(e, depth))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(ResolvedExpr::List(resolved_list))
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
