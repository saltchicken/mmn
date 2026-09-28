mod syntax;

use chumsky::Parser;
use std::env;
use std::fs;
use syntax::parser;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        return Err(format!("Usage: {} <input_file>", args[0]).into());
    }

    let filename = &args[1];
    let file_contents = fs::read_to_string(filename)?;

    // PASS 1: Parse (Text -> AST)
    let scene = parser::scene_parser()
        .parse(&file_contents)
        .into_result()
        .map_err(|errs| format!("Syntax Error: {:?}", errs))?;

    println!("--- Pass 1 Results (Raw AST) ---");
    println!("Configs: {:?}", scene.configs);
    println!("Aliases: {:?}", scene.aliases);
    println!("Expressions:");
    for expr in &scene.expressions {
        println!("  {:?}", expr);
    }
    println!();

    // PASS 2: Resolution (Expand aliases safely)
    let resolved_expressions = scene.resolve_all()?;

    println!("--- Pass 2 Results (Resolved AST) ---");
    for expr in &resolved_expressions {
        println!("{:?}", expr);
    }
    println!();

    Ok(())
}
