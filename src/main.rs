mod syntax;

use chumsky::Parser;
use std::env;
use std::fs;
use syntax::parser;
use syntax::traversal::TraversalContext;

// Return a generic Error from main
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        // returning an error string directly
        return Err(format!("Usage: {} <input_file>", args[0]).into());
    }

    let filename = &args[1];
    // Use ? to automatically handle file read errors
    let file_contents = fs::read_to_string(filename)?;

    // PASS 1: Parse
    let scene = parser::scene_parser()
        .parse(&file_contents)
        .into_result()
        .map_err(|errs| format!("Syntax Error: {:?}", errs))?;

    println!("--- AST After Pass 1 (Parse) ---\n{:?}\n", scene);

    // PASS 2: Resolution (Returns our new safe types)
    let resolved_expressions = scene.resolve_all()?;

    println!(
        "--- AST After Pass 2 (Resolution) ---\n{:?}\n",
        resolved_expressions
    );

    // PASS 3: Traversal / Execution
    let mut ctx = TraversalContext::default();

    // Apply configurations parsed from the text file (#root, #scale) dynamically
    ctx.apply_scene_configs(&scene.configs)?;

    ctx.walk_all(&resolved_expressions);

    println!("--- Pass 3 Results (MIDI Track) ---");
    println!("Root Note: {}", ctx.root_note);
    println!("Scale Intervals: {:?}", ctx.scale);
    println!("BPM: {}", ctx.bpm);
    println!("Total Ticks: {}", ctx.current_tick);

    for note in &ctx.track {
        println!(
            "Tick {:>4} -> Note On: Pitch {:>3}, Vel {:>3} (Duration: {})",
            note.start_tick, note.pitch, note.velocity, note.duration
        );
    }

    Ok(())
}
