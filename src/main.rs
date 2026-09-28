mod syntax;

use std::env;
use std::fs;
use std::process;
use chumsky::Parser;
use syntax::parser;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <input_file>", args[0]);
        process::exit(1);
    }

    let file_contents = fs::read_to_string(&args[1])?;

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
    
    // PASS 2: Resolution (Expand aliases safely)
    let resolved_expressions = scene.resolve_all()?;

    println!("\n--- Pass 2 Results (Resolved AST) ---");
    for expr in &resolved_expressions {
        println!("{:?}", expr);
    }

    Ok(())
}
