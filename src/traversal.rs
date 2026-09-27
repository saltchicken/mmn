use crate::ast::Expr;

/// Holds the state of the traversal as we walk down the tree.
#[derive(Debug, Default)]
pub struct TraversalContext {
    pub total_sum: i32,
    pub output_buffer: Vec<i32>,
}

impl TraversalContext {
    pub fn new() -> Self {
        Self::default()
    }

    /// The main entry point for traversing an AST node.
    pub fn walk(&mut self, expr: &Expr) {
        match expr {
            Expr::Num(n) => self.process_num(*n),
            Expr::List(list) => self.process_list(list),
        }
    }

    fn process_num(&mut self, n: i32) {
        // Mutate context state when hitting a leaf node
        self.total_sum += n;
        self.output_buffer.push(n);
    }

    fn process_list(&mut self, list: &[Expr]) {
        // Recursively route branch nodes back through the main walker
        for child in list {
            self.walk(child);
        }
    }
}
