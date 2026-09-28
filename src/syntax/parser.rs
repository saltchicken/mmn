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
    let comment = just("//").ignore_then(none_of('\n').repeated()).ignored();
    
    whitespace.or(comment).repeated().ignored()
}

pub fn expr_parser<'a>() -> impl Parser<'a, &'a str, Expr, extra::Err<Rich<'a, char>>> + Clone {
    recursive(|expr| {
        // Parses "@3", "@4", etc. Defaults to 1 if not present.
        let weight = just('@')
            .ignore_then(text::int(10).map(|s: &str| s.parse::<u32>().unwrap()))
            .or_not()
            .map(|w| w.unwrap_or(1));

        let interval = just('-')
            .or_not()
            .then(text::int(10))
            .then(weight.clone()) // Attach weight
            .map(|((minus, s), w): ((Option<char>, &str), u32)| {
                let val: i32 = s.parse().unwrap();
                Expr::Interval { 
                    index: if minus.is_some() { -val } else { val }, 
                    weight: w 
                }
            })
            .padded_by(padding());

        let string = just('"')
            .ignore_then(none_of('"').repeated().collect::<String>())
            .then_ignore(just('"'))
            .map(Expr::Str)
            .padded_by(padding());

        // 1. Chords: Elements separated by commas, optional weight at the end
        let comma_list = expr
            .clone()
            .separated_by(just(',').padded_by(padding()))
            .at_least(2)
            .collect::<Vec<_>>()
            .delimited_by(
                just('[').padded_by(padding()), 
                just(']').padded_by(padding())
            )
            .then(weight.clone()) // Attach weight
            .map(|(elements, w)| Expr::Chord { elements, weight: w })
            .padded_by(padding());

        // 2. Sequences: Elements separated by spaces, optional weight at the end
        let space_list = expr
            .clone()
            .repeated()
            .collect::<Vec<_>>()
            .delimited_by(
                just('[').padded_by(padding()), 
                just(']').padded_by(padding())
            )
            .then(weight.clone()) // Attach weight
            .map(|(elements, w)| Expr::Pattern { elements, weight: w })
            .padded_by(padding());

        let reference = text::ascii::ident()
            .then(just('#').or_not())
            .map(|(id, hash): (&str, Option<char>)| {
                let mut s = id.to_string();
                if let Some(h) = hash {
                    s.push(h);
                }
                
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
            .padded_by(padding());

        // Attach weight to rests
        let rest = just('~')
            .ignore_then(weight.clone())
            .map(|w| Expr::Rest { weight: w })
            .padded_by(padding());

        interval.or(string).or(comma_list).or(space_list).or(reference).or(rest)
    })
}

pub fn scene_parser<'a>() -> impl Parser<'a, &'a str, Scene, extra::Err<Rich<'a, char>>> {
    let expr = expr_parser();

    // Changed: Configs now consume everything up to the newline as a single string
    let config = just('#')
        .ignore_then(text::ascii::ident())
        .then(none_of('\n').repeated().collect::<String>())
        .padded_by(padding())
        .map(|(name, val)| {
            // Strip out inline comments if any exist on the config line
            let cleaned_val = if let Some(idx) = val.find("//") {
                val[..idx].trim().to_string()
            } else {
                val.trim().to_string()
            };
            TopLevelItem::Config(name.to_string(), Expr::Str(cleaned_val))
        });

    let alias_assign = text::ascii::ident()
        .padded_by(padding())
        .then_ignore(just('=').padded_by(padding()))
        .then(expr.clone())
        .map(|(name, e)| TopLevelItem::Alias(name.to_string(), e));

    let top_level_item = config
        .or(alias_assign)
        .or(expr.map(TopLevelItem::Expr))
        .padded_by(padding());

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
        .then_ignore(end().padded_by(padding()))
}
