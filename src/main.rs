mod syntax;

use chumsky::Parser;
use std::env;
use std::fs;
use syntax::parser;
use syntax::traversal::TraversalContext;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        eprintln!("Usage: {} <input_file>", args[0]);
        std::process::exit(1);
    }

    let filename = &args[1];
    let file_contents = fs::read_to_string(filename)
        .unwrap_or_else(|err| panic!("Failed to read file '{}': {}", filename, err));

    // Process the file line by line
    for (line_num, line) in file_contents.lines().enumerate() {
        let input = line.trim();
        
        // Skip empty lines
        if input.is_empty() {
            continue;
        }

        println!("--- Line {}: {} ---", line_num + 1, input);

        // 1. Parse (Syntax -> AST)
        match parser::expr_parser().parse(input).into_result() {
            Ok(ast) => {
                println!("Success parsing '{}':\n{:?}\n", input, ast);
                
                // 2. Setup Context
                let mut ctx = TraversalContext::new();

                // 3. Execute (AST + Context -> Output)
                ctx.walk(&ast);

                // 4. Read Results
                println!("Sum: {}", ctx.total_sum);
                println!("Buffer: {:?}\n", ctx.output_buffer);
            }
            Err(errs) => {
                println!("Error parsing:\n{:?}\n", errs);
            }
        }
    }
}
