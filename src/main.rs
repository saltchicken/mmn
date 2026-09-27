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

    // PASS 1: Parse the entire file into a single AST
    let mut program = match parser::program_parser().parse(&file_contents).into_result() {
        Ok(ast) => ast,
        Err(errs) => {
            println!("Syntax Error:\n{:?}", errs);
            std::process::exit(1);
        }
    };

    println!("--- AST After Pass 1 (Parse) ---");
    println!("{:?}\n", program);

    // PASS 2: Expand all macros globally
    if let Err(e) = program.expand_all_refs() {
        println!("Resolution Error: {}", e);
        std::process::exit(1);
    }

    println!("--- AST After Pass 2 (Resolution) ---");
    println!("{:?}\n", program.expressions);

    // PASS 3: Traversal / Execution
    let mut ctx = TraversalContext::new();
    ctx.walk_program(&program);

    println!("--- Pass 3 Results ---");
    println!("Sum: {}", ctx.total_sum);
    println!("Buffer: {:?}", ctx.output_buffer);
}
