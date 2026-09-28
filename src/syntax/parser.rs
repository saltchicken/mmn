use super::ast::{Expr, Scene};
use chumsky::prelude::*;
use std::collections::HashMap;

enum TopLevelItem {
    Config(String, Expr),
    Alias(String, Expr),
    Expr(Expr),
}

pub fn expr_parser<'a>() -> impl Parser<'a, &'a str, Expr, extra::Err<Rich<'a, char>>> + Clone {
    recursive(|expr| {
        let num = just('-')
            .or_not()
            .then(text::int(10))
            .map(|(minus, s): (Option<char>, &str)| {
                let val: i32 = s.parse().unwrap();
                Expr::Num(if minus.is_some() { -val } else { val })
            })
            .padded();

        let list = expr
            .clone()
            .repeated()
            .collect::<Vec<_>>()
            .delimited_by(just('['), just(']'))
            .map(Expr::List)
            .padded();

        let reference = text::ascii::ident()
            .then(just('#').or(just('b')).or(just('B')).or_not())
            .map(|(id, accidental): (&str, Option<char>)| {
                let mut s = id.to_string();
                if let Some(a) = accidental {
                    s.push(a);
                }
                Expr::Ref(s)
            })
            .padded();

        num.or(list).or(reference)
    })
}

pub fn scene_parser<'a>() -> impl Parser<'a, &'a str, Scene, extra::Err<Rich<'a, char>>> {
    let expr = expr_parser();

    let config = just('#')
        .ignore_then(text::ascii::ident())
        .padded()
        .then(expr.clone())
        .map(|(name, e)| TopLevelItem::Config(name.to_string(), e));

    let alias_assign = text::ascii::ident()
        .padded()
        .then_ignore(just('=').padded())
        .then(expr.clone())
        .map(|(name, e)| TopLevelItem::Alias(name.to_string(), e));

    // Try config, then alias, then expression
    let top_level_item = config
        .or(alias_assign)
        .or(expr.map(TopLevelItem::Expr))
        .padded();

    // Consume the entire file, fold into a Scene AST
    top_level_item
        .repeated()
        .collect::<Vec<_>>()
        .map(|items| {
            let mut configs = HashMap::new();
            let mut aliases = HashMap::new();
            let mut expressions = Vec::new();

            for item in items {
                match item {
                    TopLevelItem::Config(name, expr) => {
                        // Store the configuration raw
                        configs.insert(name.to_uppercase(), expr);
                    }
                    TopLevelItem::Alias(name, expr) => {
                        aliases.insert(name, expr);
                    }
                    TopLevelItem::Expr(expr) => {
                        expressions.push(expr);
                    }
                }
            }

            Scene {
                configs,
                aliases,
                expressions,
            }
        })
        .then_ignore(end())
}
