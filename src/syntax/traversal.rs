use super::ast::{Expr, Program};

#[derive(Debug, Default)]
pub struct TraversalContext {
    pub total_sum: i32,
    pub output_buffer: Vec<i32>,
}

impl TraversalContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn walk_program(&mut self, program: &Program) {
        for expr in &program.expressions {
            self.walk(expr);
        }
    }

    pub fn walk(&mut self, expr: &Expr) {
        match expr {
            Expr::Num(n) => self.process_num(*n),
            Expr::List(list) => self.process_list(list),
            Expr::Ref(name) => panic!("Unresolved reference reached execution: {}", name),
        }
    }

    fn process_num(&mut self, n: i32) {
        self.total_sum += n;
        self.output_buffer.push(n);
    }

    fn process_list(&mut self, list: &[Expr]) {
        for child in list {
            self.walk(child);
        }
    }
}
