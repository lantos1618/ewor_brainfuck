use ewor_brainfuck::{Mode, BF};
use std::env;
use std::fs;

fn main() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        return Err("Usage: bf [-a] [-d<num>] <source_file>".to_string());
    }

    let mut mode = Mode::BF;
    let mut dump_cells = None;
    let mut source_file = None;

    let mut i = 1;
    while i < args.len() {
        if args[i] == "-a" {
            mode = Mode::BFA;
        } else if args[i].starts_with("-d") {
            dump_cells = Some(
                args[i][2..]
                    .parse::<usize>()
                    .map_err(|_| "Invalid dump number".to_string())?,
            );
        } else {
            source_file = Some(&args[i]);
        }
        i += 1;
    }

    let source_file = source_file.ok_or("No source file provided".to_string())?;
    let code = fs::read_to_string(source_file).map_err(|e| format!("Error reading file: {}", e))?;

    let mut bf = BF::new(&code, mode);
    let result = bf.run();

    if let Some(n) = dump_cells {
        eprintln!("\nFirst {} cells:", n);
        eprintln!("{:?}", bf.dump_cells(n));
    }

    result
}
