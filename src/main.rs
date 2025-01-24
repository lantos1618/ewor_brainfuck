use std::env;
use std::fs;

use ewor_brainfuck::bf::Mode;
use ewor_brainfuck::bf::BF;
use ewor_brainfuck::bfl::BFLCompiler;
use ewor_brainfuck::bfl::BFLNode;

fn main() -> Result<(), String> {
    let mut compiler = BFLCompiler::new();
    // let message = "Hello, World!\n";
    // syscall(2, 1, 14, message)
    let program = BFLNode::Block(vec![
        BFLNode::Assign(
            "message".to_string(),
            Box::new(BFLNode::String("Hello, World!\n".to_string())),
        ),
        BFLNode::Syscall(
            Box::new(BFLNode::Number(1)),
            vec![
                BFLNode::Number(1),
                BFLNode::Variable("message".to_string()),
                BFLNode::Number(14),
            ],
        ),
    ]);

    compiler.compile(&program).unwrap();
    let bf_code = compiler.get_output();

    println!("\nGenerated BF code:");
    println!("{}", bf_code);
    println!("\nTrying to run...");

    let mut bf = BF::new(bf_code, Mode::BFA);
    println!("\nFirst 20 cells before execution:");
    println!("{:?}", bf.dump_cells(20));

    bf.run().unwrap();

    Ok(())
    // let args: Vec<String> = env::args().collect();

    // if args.len() < 2 {
    //     return Err("Usage: bf [-a] [-d<num>] <source_file>".to_string());
    // }

    // let mut mode = Mode::BF;
    // let mut dump_cells = None;
    // let mut source_file = None;

    // let mut i = 1;
    // while i < args.len() {
    //     if args[i] == "-a" {
    //         mode = Mode::BFA;
    //     } else if args[i].starts_with("-d") {
    //         dump_cells = Some(
    //             args[i][2..]
    //                 .parse::<usize>()
    //                 .map_err(|_| "Invalid dump number".to_string())?,
    //         );
    //     } else {
    //         source_file = Some(&args[i]);
    //     }
    //     i += 1;
    // }

    // let source_file = source_file.ok_or("No source file provided".to_string())?;
    // let code = fs::read_to_string(source_file).map_err(|e| format!("Error reading file: {}", e))?;

    // let mut bf = BF::new(&code, mode);
    // let result = match bf.run() {
    //     Ok(()) => Ok(()),
    //     Err(e) => Err(e.to_string()),
    // };

    // if let Some(n) = dump_cells {
    //     eprintln!("\nFirst {} cells:", n);
    //     eprintln!("{:?}", bf.dump_cells(n));
    // }

    // result
}
