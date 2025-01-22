use bfa::{Mode, BF};
use std::env;
use std::fs;

fn main() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        return Err("Usage: bf [-a] <source_file>".to_string());
    }

    let mut mode = Mode::BF;
    let source_file = if args[1] == "-a" {
        mode = Mode::BFA;
        if args.len() < 3 {
            return Err("Usage: bf [-a] <source_file>".to_string());
        }
        &args[2]
    } else {
        &args[1]
    };

    let code = fs::read_to_string(source_file).map_err(|e| format!("Error reading file: {}", e))?;

    let mut bf = BF::new(&code, mode);
    bf.run()
}
