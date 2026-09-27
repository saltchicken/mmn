use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Num(i32),
    List(Vec<Expr>),
    Ref(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub macros: HashMap<String, Expr>,
    pub expressions: Vec<Expr>,
}

impl Program {
    /// Pass 2: Resolves all references before execution begins.
    pub fn expand_all_refs(&mut self) -> Result<(), String> {
        // Step A: Expand macros that reference other macros
        let env = self.macros.clone();
        for expr in self.macros.values_mut() {
            expr.expand_refs(&env, 0)?;
        }

        // Step B: Use the fully expanded macros to resolve the main execution expressions
        let fully_expanded_env = self.macros.clone();
        for expr in &mut self.expressions {
            expr.expand_refs(&fully_expanded_env, 0)?;
        }

        Ok(())
    }
}

impl Expr {
    pub fn expand_refs(&mut self, env: &HashMap<String, Expr>, depth: usize) -> Result<(), String> {
        if depth > 32 {
            return Err("Max macro expansion depth exceeded (circular reference?)".to_string());
        }

        match self {
            Expr::Ref(name) => {
                if let Some(resolved) = env.get(name) {
                    let mut cloned = resolved.clone();
                    cloned.expand_refs(env, depth + 1)?;
                    *self = cloned;
                } else {
                    return Err(format!("Unresolved macro: '{}'", name));
                }
            }
            Expr::List(list) => {
                for el in list {
                    el.expand_refs(env, depth)?;
                }
            }
            Expr::Num(_) => {}
        }
        Ok(())
    }
}
