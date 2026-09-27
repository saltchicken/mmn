use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Num(i32),
    List(Vec<Expr>),
    Ref(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ResolvedExpr {
    Num(i32),
    List(Vec<ResolvedExpr>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Scene {
    pub aliases: HashMap<String, Expr>,
    pub expressions: Vec<Expr>,
}

impl Scene {
    /// Pass 2: Converts a Scene of raw Exprs into a Vec of safely Executable Exprs.
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
            Expr::Num(n) => Ok(ResolvedExpr::Num(*n)),
            Expr::List(list) => {
                let resolved_list = list
                    .iter()
                    .map(|e| self.resolve_expr(e, depth))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(ResolvedExpr::List(resolved_list))
            }
            Expr::Ref(name) => {
                let alias_expr = self.aliases.get(name)
                    .ok_or_else(|| format!("Unresolved alias: '{}'", name))?;
                // Recursively resolve the alias
                self.resolve_expr(alias_expr, depth + 1)
            }
        }
    }
}
