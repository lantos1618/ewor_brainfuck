use ewor_brainfuck::bf::{BF, Mode};
use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <brainfuck_file.bf>", args[0]);
        std::process::exit(1);
    }

    let filename = &args[1];
    let bf_code = match fs::read_to_string(filename) {
        Ok(code) => code,
        Err(e) => {
            eprintln!("Error reading file {}: {}", filename, e);
            std::process::exit(1);
        }
    };

    println!("Running brainfuck program from {}...", filename);
    let mut bf = BF::new(&bf_code, Mode::BF);
    
    match bf.run() {
        Ok(_) => println!("\nProgram completed successfully"),
        Err(e) => {
            eprintln!("\nError running brainfuck program: {}", e);
            std::process::exit(1);
        }
    }
} 