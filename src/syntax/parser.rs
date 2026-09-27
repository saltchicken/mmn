use super::ast::{Expr, Program};
use chumsky::prelude::*;
use std::collections::HashMap;

enum TopLevelItem {
    Macro(String, Expr),
    Expr(Expr),
}

pub fn expr_parser<'a>() -> impl Parser<'a, &'a str, Expr, extra::Err<Rich<'a, char>>> + Clone {
    recursive(|expr| {
        let num = text::int(10)
            .map(|s: &str| Expr::Num(s.parse().unwrap()))
            .padded();

        let list = expr
            .clone()
            .repeated()
            .collect::<Vec<_>>()
            .delimited_by(just('['), just(']'))
            .map(Expr::List)
            .padded();

        let reference = text::ascii::ident()
            .map(|s: &str| Expr::Ref(s.to_string()))
            .padded();

        num.or(list).or(reference)
    })
}

pub fn program_parser<'a>() -> impl Parser<'a, &'a str, Program, extra::Err<Rich<'a, char>>> {
    let expr = expr_parser();

    let macro_assign = text::ascii::ident()
        .padded()
        .then_ignore(just('=').padded())
        .then(expr.clone())
        .map(|(name, e)| TopLevelItem::Macro(name.to_string(), e));

    let top_level_item = macro_assign.or(expr.map(TopLevelItem::Expr)).padded();

    // Consume the entire file, fold into a Program AST
    top_level_item
        .repeated()
        .collect::<Vec<_>>()
        .map(|items| {
            let mut macros = HashMap::new();
            let mut expressions = Vec::new();

            for item in items {
                match item {
                    TopLevelItem::Macro(name, expr) => {
                        macros.insert(name, expr);
                    }
                    TopLevelItem::Expr(expr) => {
                        expressions.push(expr);
                    }
                }
            }

            Program {
                macros,
                expressions,
            }
        })
        .then_ignore(end())
}
