use super::ast::{Expr, Scene};
use chumsky::prelude::*;
use std::collections::HashMap;

enum TopLevelItem {
    Config(String, Expr),
    Alias(String, Expr),
    Expr(Expr),
}

/// Helper to map a musical pitch class string to a 0-11 integer
fn match_pitch_class(note: &str) -> Option<u8> {
    match note.to_uppercase().as_str() {
        "C" => Some(0),
        "C#" | "DB" => Some(1),
        "D" => Some(2),
        "D#" | "EB" => Some(3),
        "E" => Some(4),
        "F" => Some(5),
        "F#" | "GB" => Some(6),
        "G" => Some(7),
        "G#" | "AB" => Some(8),
        "A" => Some(9),
        "A#" | "BB" => Some(10),
        "B" => Some(11),
        _ => None,
    }
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
            let mut root_class = None;
            let mut octave = None;
            
            let mut scale = None;
            let mut aliases = HashMap::new();
            let mut expressions = Vec::new();

            for item in items {
                match item {
                    TopLevelItem::Config(name, expr) => {
                        let name_upper = name.to_uppercase();
                        if name_upper == "ROOT" {
                            if let Expr::Ref(s) = expr {
                                if let Some(c) = match_pitch_class(&s) {
                                    root_class = Some(c);
                                } else {
                                    println!("Warning: Invalid ROOT pitch class '{}'", s);
                                }
                            } else {
                                println!("Warning: #ROOT must be a note like C or C#");
                            }
                        } else if name_upper == "OCTAVE" {
                            if let Expr::Num(n) = expr {
                                octave = Some(n);
                            } else {
                                println!("Warning: #OCTAVE must be a number");
                            }
                        } else if name_upper == "SCALE" {
                            if let Expr::Ref(s) = expr {
                                scale = Some(s);
                            } else {
                                println!("Warning: #SCALE must be a string like minor or major");
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

            // Calculate the final MIDI root note if ROOT or OCTAVE was specified.
            // If one is missing, use standard defaults (C and Octave 4).
            let mut root_note = None;
            if root_class.is_some() || octave.is_some() {
                let c = root_class.unwrap_or(0); // Default to C
                let o = octave.unwrap_or(4);     // Default to octave 4
                
                // C4 = 60 => (4 + 1) * 12 + 0 = 60
                let midi_note = (o + 1) * 12 + (c as i32);
                
                if (0..=127).contains(&midi_note) {
                    root_note = Some(midi_note as u8);
                } else {
                    println!("Warning: Calculated root note {} is out of MIDI range (0-127). Falling back to default.", midi_note);
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
