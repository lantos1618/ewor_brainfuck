use ewor_brainfuck::bfl::{BFLCompiler, BFLNode};
use ewor_brainfuck::{Mode, BF};

fn main() {
    // Create a program that writes "Hello, World!" using syscalls
    let program = BFLNode::Block(vec![
        // Store "Hello, World!\n" in memory starting at position 8
        BFLNode::Assign("msg_h".to_string(), Box::new(BFLNode::Number(72))), // H
        BFLNode::Assign("msg_e".to_string(), Box::new(BFLNode::Number(101))), // e
        BFLNode::Assign("msg_l1".to_string(), Box::new(BFLNode::Number(108))), // l
        BFLNode::Assign("msg_l2".to_string(), Box::new(BFLNode::Number(108))), // l
        BFLNode::Assign("msg_o".to_string(), Box::new(BFLNode::Number(111))), // o
        BFLNode::Assign("msg_comma".to_string(), Box::new(BFLNode::Number(44))), // ,
        BFLNode::Assign("msg_space".to_string(), Box::new(BFLNode::Number(32))), // space
        BFLNode::Assign("msg_w".to_string(), Box::new(BFLNode::Number(87))), // W
        BFLNode::Assign("msg_o2".to_string(), Box::new(BFLNode::Number(111))), // o
        BFLNode::Assign("msg_r".to_string(), Box::new(BFLNode::Number(114))), // r
        BFLNode::Assign("msg_l3".to_string(), Box::new(BFLNode::Number(108))), // l
        BFLNode::Assign("msg_d".to_string(), Box::new(BFLNode::Number(100))), // d
        BFLNode::Assign("msg_bang".to_string(), Box::new(BFLNode::Number(33))), // !
        BFLNode::Assign("msg_nl".to_string(), Box::new(BFLNode::Number(10))), // \n
        // Write syscall: write(1, msg, 14)
        BFLNode::Syscall(
            Box::new(BFLNode::Number(1)), // write syscall
            vec![
                BFLNode::Number(1),                     // fd (stdout)
                BFLNode::Variable("msg_h".to_string()), // buffer start
                BFLNode::Number(14),                    // length
            ],
        ),
    ]);

    // Compile the program to brainfuck
    let mut compiler = BFLCompiler::new();
    compiler
        .compile(&program)
        .expect("Failed to compile program");

    // Get the generated brainfuck code
    let bf_code = compiler.get_output();
    println!("Generated Brainfuck code:");
    println!("{}", bf_code);

    // Run the brainfuck code
    println!("\nProgram output:");
    let mut bf = BF::new(bf_code, Mode::BFA);
    bf.run().expect("Failed to run program");
}
