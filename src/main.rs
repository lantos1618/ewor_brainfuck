use ewor_brainfuck::bf::{Mode, BF};
use ewor_brainfuck::bfl::{BFLCompiler, BFLNode};
use std::process::ExitCode;
use std::env;
use std::fs;

// Using Linux x86_64 syscall numbers, as the devcontainer is Debian.
const SYS_SOCKET: i32 = 41;
const SYS_BIND: i32 = 49;
const SYS_LISTEN: i32 = 50;
const SYS_ACCEPT: i32 = 43;
const SYS_READ: i32 = 0;
const SYS_WRITE: i32 = 1;
const SYS_CLOSE: i32 = 3;

// Socket constants
const AF_INET: i32 = 2;
const SOCK_STREAM: i32 = 1;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        eprintln!("Usage: {} <brainfuck_file.bf>", args[0]);
        eprintln!("   or: {} --test", args[0]);
        return ExitCode::FAILURE;
    }

    if args[1] == "--test" {
        // Run the test program
        return run_test_program();
    }

    // Read the brainfuck file
    let filename = &args[1];
    let bf_code = match fs::read_to_string(filename) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Failed to read file '{}': {}", filename, e);
            return ExitCode::FAILURE;
        }
    };

    // Determine mode based on file extension or default to BF
    let mode = if filename.ends_with(".bfa") {
        Mode::BFA
    } else {
        Mode::BF
    };

    // Run the brainfuck program
    let mut bf = BF::new(&bf_code, mode);
    if let Err(e) = bf.run() {
        eprintln!("Brainfuck program failed: {}", e);
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}

fn run_test_program() -> ExitCode {
    let mut compiler = BFLCompiler::new();

    // Simple test: just write a message to stdout
    let test_program = BFLNode::Block(vec![
        // Write a simple message to stdout
        BFLNode::Assign(
            "msg".to_string(),
            Box::new(BFLNode::String("Hello from Brainfuck!\n".to_string())),
        ),
        BFLNode::Syscall(
            Box::new(BFLNode::Number(SYS_WRITE)),
            vec![
                BFLNode::Number(1), // stdout
                BFLNode::Variable("msg".to_string()),
                BFLNode::Number(22), // length
            ],
        ),
    ]);
    
    println!("Compiling simple test...");
    if let Err(e) = compiler.compile(&test_program) {
        eprintln!("Failed to compile BFL program: {}", e);
        return ExitCode::FAILURE;
    }
    
    let bf_code = compiler.get_output();
    println!("Generated BF code length: {}", bf_code.len());

    println!("Running simple test...");
    let mut bf = BF::new(bf_code, Mode::BFA);
    if let Err(e) = bf.run() {
        eprintln!("Brainfuck program failed: {}", e);
        return ExitCode::FAILURE;
    }

    println!("Simple test completed successfully!");
    ExitCode::SUCCESS
}
