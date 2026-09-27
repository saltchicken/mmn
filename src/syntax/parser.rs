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

pub fn scene_parser<'a>() -> impl Parser<'a, &'a str, Scene, extra::Err<Rich<'a, char>>> {
    let expr = expr_parser();

    // Parses: #root 60
    let config = just('#')
        .ignore_then(text::ascii::ident())
        .padded()
        .then(expr.clone())
        .map(|(name, e)| TopLevelItem::Config(name.to_string(), e));

    // Parses: riff = [0 2 4 7]
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
            let mut root_note = None;
            let mut scale = None;
            let mut aliases = HashMap::new();
            let mut expressions = Vec::new();

            for item in items {
                match item {
                    TopLevelItem::Config(name, expr) => {
                        if name == "root" {
                            if let Expr::Num(n) = expr {
                                root_note = Some(n as u8);
                            }
                        } else if name == "scale" {
                            if let Expr::Ref(s) = expr {
                                scale = Some(s);
                            }
                        } else {
                            println!("Warning: Unknown config directive '#{}'", name);
                        }
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
                root_note,
                scale,
                aliases,
                expressions,
            }
        })
        .then_ignore(end())
}
