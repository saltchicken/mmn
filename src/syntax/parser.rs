use super::ast::{Expr, Scene};
use chumsky::prelude::*;
use std::collections::HashMap;

enum TopLevelItem {
    Config(String, Expr),
    Alias(String, Expr),
    Expr(Expr),
}

// A helper enum to capture order-independent modifiers
#[derive(Clone, Copy)]
enum Modifier {
    Velocity(u32),
    Hold(f32),
    Span(f32),
}

// Helper function to fold parsed modifiers down to a standardized tuple (velocity, length)
fn apply_modifiers(mods: Vec<Modifier>) -> (Option<u32>, f32) {
    let mut velocity = None;
    let mut length = 1.0; // represents either hold or span
    for m in mods {
        match m {
            Modifier::Velocity(v) => velocity = Some(v),
            Modifier::Hold(v) | Modifier::Span(v) => length = v,
        }
    }
    (velocity, length)
}

/// Custom padding parser that ignores both standard whitespace and single-line comments.
fn padding<'a>() -> impl Parser<'a, &'a str, (), extra::Err<Rich<'a, char>>> + Clone {
    let whitespace = any().filter(|c: &char| c.is_whitespace()).ignored();
    let comment = just("//").then_ignore(none_of('\n').repeated()).ignored();

    whitespace.or(comment).repeated().ignored()
}

pub fn expr_parser<'a>() -> impl Parser<'a, &'a str, Expr, extra::Err<Rich<'a, char>>> + Clone {
    recursive(|expr| {
        // Construct a float parser that supports integers and decimals
        let float = text::int(10)
            .then(just('.').ignore_then(text::int(10)).or_not())
            .to_slice() // Extracts the matched `&str` directly
            .try_map(|s: &str, span| {
                s.parse::<f32>().map_err(|_e| Rich::custom(span, "Invalid float value"))
            });

        // Simple hold parser for Rests
        let rest_hold = just(".hold")
            .ignore_then(just('(').padded_by(padding()))
            .ignore_then(float.clone())
            .then_ignore(just(')').padded_by(padding()))
            .or_not()
            .map(|h| h.unwrap_or(1.0));

        // Unordered modifier parser components
        let vel_mod = just(".vel")
            .ignore_then(just('(').padded_by(padding()))
            .ignore_then(text::int(10).map(|s: &str| Modifier::Velocity(s.parse().unwrap())))
            .then_ignore(just(')').padded_by(padding()));
            
        let hold_mod = just(".hold")
            .ignore_then(just('(').padded_by(padding()))
            .ignore_then(float.clone().map(|f| Modifier::Hold(f)))
            .then_ignore(just(')').padded_by(padding()));

        let span_mod = just(".span")
            .ignore_then(just('(').padded_by(padding()))
            .ignore_then(float.clone().map(|f| Modifier::Span(f)))
            .then_ignore(just(')').padded_by(padding()));

        // Separate groups: Hold applies to Intervals/Chords, Span applies to Patterns
        let hold_modifier = vel_mod.clone().or(hold_mod);
        let span_modifier = vel_mod.clone().or(span_mod);

        let interval = just('-')
            .or_not()
            .then(text::int(10))
            .then(hold_modifier.clone().repeated().collect::<Vec<_>>())
            .map(|((minus, s), mods): ((Option<char>, &str), Vec<Modifier>)| {
                let val: i32 = s.parse().unwrap();
                let (velocity, hold) = apply_modifiers(mods);

                Expr::Interval {
                    index: if minus.is_some() { -val } else { val },
                    velocity,
                    hold,
                }
            })
            .padded_by(padding());

        let string = just('"')
            .ignore_then(none_of('"').repeated().collect::<String>())
            .then_ignore(just('"'))
            .map(Expr::Str)
            .padded_by(padding());

        // 1. Chords: Elements separated by commas (uses hold)
        let comma_list = expr
            .clone()
            .separated_by(just(',').padded_by(padding()))
            .at_least(2)
            .collect::<Vec<_>>()
            .delimited_by(
                just('[').padded_by(padding()),
                just(']').padded_by(padding()),
            )
            .then(hold_modifier.clone().repeated().collect::<Vec<_>>())
            .map(|(elements, mods): (Vec<Expr>, Vec<Modifier>)| {
                let (velocity, hold) = apply_modifiers(mods);
                Expr::Chord {
                    elements,
                    velocity,
                    hold,
                }
            })
            .padded_by(padding());

        // 2. Sequences: Elements separated by spaces (uses span)
        let space_list = expr
            .clone()
            .repeated()
            .collect::<Vec<_>>()
            .delimited_by(
                just('[').padded_by(padding()),
                just(']').padded_by(padding()),
            )
            .then(span_modifier.clone().repeated().collect::<Vec<_>>())
            .map(|(elements, mods): (Vec<Expr>, Vec<Modifier>)| {
                let (velocity, span) = apply_modifiers(mods);
                Expr::Pattern {
                    elements,
                    velocity,
                    span,
                }
            })
            .padded_by(padding());

        let reference = text::ascii::ident()
            .then(just('#').or_not())
            .map(|(id, hash): (&str, Option<char>)| {
                let mut s = id.to_string();
                if let Some(h) = hash {
                    s.push(h);
                }

                let is_symbol = matches!(
                    s.to_uppercase().as_str(),
                    "C" | "C#"
                        | "DB"
                        | "D"
                        | "D#"
                        | "EB"
                        | "E"
                        | "F"
                        | "F#"
                        | "GB"
                        | "G"
                        | "G#"
                        | "AB"
                        | "A"
                        | "A#"
                        | "BB"
                        | "B"
                );

                if is_symbol {
                    Expr::Symbol(s)
                } else {
                    Expr::Ident(s)
                }
            })
            .padded_by(padding());

        let rest = just('~')
            .ignore_then(rest_hold)
            .map(|hold| Expr::Rest { hold })
            .padded_by(padding());

        interval
            .or(string)
            .or(comma_list)
            .or(space_list)
            .or(reference)
            .or(rest)
    })
}

pub fn scene_parser<'a>() -> impl Parser<'a, &'a str, Scene, extra::Err<Rich<'a, char>>> {
    let expr = expr_parser();

    let config = just('#')
        .ignore_then(text::ascii::ident())
        .then(none_of('\n').repeated().collect::<String>())
        .padded_by(padding())
        .map(|(name, val)| {
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
            let mut scene = Scene {
                configs: HashMap::new(),
                aliases: HashMap::new(),
                expressions: Vec::new(),
            };

            for item in items {
                match item {
                    TopLevelItem::Config(name, expr) => {
                        scene.configs.insert(name.to_uppercase(), expr);
                    }
                    TopLevelItem::Alias(name, expr) => {
                        scene.aliases.insert(name, expr);
                    }
                    TopLevelItem::Expr(expr) => {
                        scene.expressions.push(expr);
                    }
                }
            }
            scene
        })
        .then_ignore(end().padded_by(padding()))
}
