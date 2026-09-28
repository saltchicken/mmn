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

        let string = just('"')
            .ignore_then(none_of('"').repeated().collect::<String>())
            .then_ignore(just('"'))
            .map(Expr::Str)
            .padded();

        let list = expr
            .clone()
            .repeated()
            .collect::<Vec<_>>()
            .delimited_by(just('['), just(']'))
            .map(Expr::List)
            .padded();

        let reference = text::ascii::ident()
            .then(just('#').or_not()) // 'b' or 'B' is naturally captured by ident() earlier
            .map(|(id, hash): (&str, Option<char>)| {
                let mut s = id.to_string();
                if let Some(h) = hash {
                    s.push(h);
                }
                
                // Discriminate between musical symbols (e.g. C#, Db) and generic identifiers
                let is_symbol = match s.to_uppercase().as_str() {
                    "C" | "C#" | "DB" | "D" | "D#" | "EB" | "E" | "F" | "F#" | "GB" | "G" | "G#" | "AB" | "A" | "A#" | "BB" | "B" => true,
                    _ => false,
                };

                if is_symbol {
                    Expr::Symbol(s)
                } else {
                    Expr::Ident(s)
                }
            })
            .padded();

        num.or(string).or(list).or(reference)
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
