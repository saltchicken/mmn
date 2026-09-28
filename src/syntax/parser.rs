use super::ast::{Expr, Scene};
use chumsky::prelude::*;
use std::collections::HashMap;

enum TopLevelItem {
    Config(String, Expr),
    Alias(String, Expr),
    Expr(Expr),
}

/// Custom padding parser that ignores both standard whitespace and single-line comments.
fn padding<'a>() -> impl Parser<'a, &'a str, (), extra::Err<Rich<'a, char>>> + Clone {
    let whitespace = any().filter(|c: &char| c.is_whitespace()).ignored();
    let comment = just("//").then_ignore(none_of('\n').repeated()).ignored();
    
    whitespace.or(comment).repeated().ignored()
}

pub fn expr_parser<'a>() -> impl Parser<'a, &'a str, Expr, extra::Err<Rich<'a, char>>> + Clone {
    recursive(|expr| {
        let weight = just('@')
            .ignore_then(text::int(10).map(|s: &str| s.parse::<u32>().unwrap()))
            .or_not()
            .map(|w| w.unwrap_or(1));

        let interval = just('-')
            .or_not()
            .then(text::int(10))
            .then(weight.clone())
            // Add the explicit type annotation back here:
            .map(|((minus, s), weight): ((Option<char>, &str), u32)| {
                let val: i32 = s.parse().unwrap();
                Expr::Interval { 
                    index: if minus.is_some() { -val } else { val }, 
                    weight 
                }
            })
            .padded_by(padding());

        let string = just('"')
            .ignore_then(none_of('"').repeated().collect::<String>())
            .then_ignore(just('"'))
            .map(Expr::Str)
            .padded_by(padding());

        // 1. Chords: Elements separated by commas
        let comma_list = expr.clone()
            .separated_by(just(',').padded_by(padding()))
            .at_least(2)
            .collect::<Vec<_>>()
            .delimited_by(just('[').padded_by(padding()), just(']').padded_by(padding()))
            .then(weight.clone())
            .map(|(elements, weight)| Expr::Chord { elements, weight })
            .padded_by(padding());

        // 2. Sequences: Elements separated by spaces
        let space_list = expr.clone()
            .repeated()
            .collect::<Vec<_>>()
            .delimited_by(just('[').padded_by(padding()), just(']').padded_by(padding()))
            .then(weight.clone())
            .map(|(elements, weight)| Expr::Pattern { elements, weight })
            .padded_by(padding());

        let reference = text::ascii::ident()
            .then(just('#').or_not())
            // Add the explicit type annotation back here:
            .map(|(id, hash): (&str, Option<char>)| {
                let mut s = id.to_string();
                if let Some(h) = hash { s.push(h); }
                
                // Simplified using the matches! macro
                let is_symbol = matches!(
                    s.to_uppercase().as_str(),
                    "C" | "C#" | "DB" | "D" | "D#" | "EB" | "E" | "F" | "F#" | 
                    "GB" | "G" | "G#" | "AB" | "A" | "A#" | "BB" | "B"
                );

                if is_symbol { Expr::Symbol(s) } else { Expr::Ident(s) }
            })
            .padded_by(padding());

        let rest = just('~')
            .ignore_then(weight) // 'weight' is already cloned where needed above
            .map(|weight| Expr::Rest { weight })
            .padded_by(padding());

        interval.or(string).or(comma_list).or(space_list).or(reference).or(rest)
    })
}

pub fn scene_parser<'a>() -> impl Parser<'a, &'a str, Scene, extra::Err<Rich<'a, char>>> {
    let expr = expr_parser();

    let config = just('#')
        .ignore_then(text::ascii::ident())
        .then(none_of('\n').repeated().collect::<String>())
        .padded_by(padding())
        .map(|(name, val)| {
            // Simplified string manipulation
            let cleaned_val = val.split("//").next().unwrap_or(&val).trim().to_string();
            TopLevelItem::Config(name.to_string(), Expr::Str(cleaned_val))
        });

    let alias_assign = text::ascii::ident()
        .padded_by(padding())
        .then_ignore(just('=').padded_by(padding()))
        .then(expr.clone())
        .map(|(name, e)| TopLevelItem::Alias(name.to_string(), e));

    config
        .or(alias_assign)
        .or(expr.map(TopLevelItem::Expr))
        .padded_by(padding())
        .repeated()
        .collect::<Vec<_>>()
        .map(|items| {
            // Initialize Scene directly
            let mut scene = Scene {
                configs: HashMap::new(),
                aliases: HashMap::new(),
                expressions: Vec::new(),
            };

            for item in items {
                match item {
                    TopLevelItem::Config(name, expr) => { scene.configs.insert(name.to_uppercase(), expr); }
                    TopLevelItem::Alias(name, expr) => { scene.aliases.insert(name, expr); }
                    TopLevelItem::Expr(expr) => { scene.expressions.push(expr); }
                }
            }
            scene
        })
        .then_ignore(end().padded_by(padding()))
}
