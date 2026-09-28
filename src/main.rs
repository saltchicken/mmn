mod syntax;
mod midi; // Bring in the new codegen backend

use chumsky::Parser;
use std::env;
use std::fs;
use syntax::parser;
use syntax::traversal::TraversalContext;
use midi::generate_midi;

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

    // PASS 2: Resolution (Expand aliases safely)
    let resolved_expressions = scene.resolve_all()?;

    // PASS 3: Traversal (AST -> Generic IR Sequence)
    let mut ctx = TraversalContext::default();
    ctx.walk_all(&resolved_expressions);

    // PASS 4: Backend / Codegen (Generic IR + Configs -> Concrete MIDI Data)
    let track_data = generate_midi(&scene.configs, &ctx.sequence)?;

    println!("--- Pass 4 Results (MIDI Track) ---");
    println!("Root Note: {}", track_data.root_note);
    println!("Scale Intervals: {:?}", track_data.scale);
    println!("BPM: {}", track_data.bpm);
    
    for note in &track_data.notes {
        println!(
            "Tick {:>4} -> Note On: Pitch {:>3}, Vel {:>3} (Duration: {})",
            note.start_tick, note.pitch, note.velocity, note.duration
        );
    }

    Ok(())
}
