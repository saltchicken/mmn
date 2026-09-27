use super::ast::ResolvedExpr;

#[derive(Debug, Default)]
pub struct TraversalContext {
    pub total_sum: i32,
    pub output_buffer: Vec<i32>,
}

impl TraversalContext {
    pub fn walk_all(&mut self, expressions: &[ResolvedExpr]) {
        for expr in expressions {
            self.walk(expr);
        }
    }

    pub fn walk(&mut self, expr: &ResolvedExpr) {
        match expr {
            ResolvedExpr::Num(n) => {
                self.total_sum += n;
                self.output_buffer.push(*n);
            }
            ResolvedExpr::List(list) => {
                for child in list {
                    self.walk(child);
                }
            }
        }
    }
}
