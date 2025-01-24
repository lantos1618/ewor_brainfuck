use std::env;
use std::fs;

use ewor_brainfuck::bf::{Mode, BF};
use ewor_brainfuck::bfl::{BFLCompiler, BFLNode};

fn main() -> Result<(), String> {
    let mut compiler = BFLCompiler::new();

    let program = BFLNode::Block(vec![
        // Store the message
        BFLNode::Assign(
            "message".to_string(),
            Box::new(BFLNode::String("Hello, World!\n".to_string())),
        ),
        // Make write syscall (1) with fd 1 (stdout) and our message
        BFLNode::Syscall(
            Box::new(BFLNode::Number(1)), // write syscall
            vec![
                BFLNode::Number(1),                       // stdout fd
                BFLNode::Variable("message".to_string()), // buffer
                BFLNode::Number(14),                      // length of "Hello, World!\n"
            ],
        ),
    ]);

    compiler.compile(&program)?;
    let bf_code = compiler.get_output();
    println!("\nGenerated BF code:\n{}", bf_code);

    let mut bf = BF::new(bf_code, Mode::BFA);
    if let Err(e) = bf.run() {
        return Err(e.to_string());
    }

    Ok(())
}
