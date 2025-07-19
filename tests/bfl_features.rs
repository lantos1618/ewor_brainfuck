use ewor_brainfuck::bf::{BF, Mode};
use ewor_brainfuck::bfl::{BFLCompiler, BFLNode};

#[test]
fn test_bfl_if_simple_condition() {
    // Test if with simple number condition (always true)
    let mut compiler = BFLCompiler::new();
    let program = BFLNode::Block(vec![
        BFLNode::If(
            Box::new(BFLNode::Number(1)), // Always true
            vec![
                BFLNode::Assign("z".to_string(), Box::new(BFLNode::Number(99))),
            ],
        ),
    ]);
    compiler.compile(&program).unwrap();
    let bf_code = compiler.get_output();
    let mut bf = BF::new(bf_code, Mode::BFA);
    bf.run().unwrap();
    let z_addr = compiler.get_variable_address("z").unwrap();
    println!("z address: {}, z value: {}", z_addr, bf.dump_cells(z_addr + 1)[z_addr]);
    assert_eq!(bf.dump_cells(z_addr + 1)[z_addr], 99);
}

#[test]
fn test_bfl_if() {
    let mut compiler = BFLCompiler::new();
    let program = BFLNode::Block(vec![
        BFLNode::Assign("x".to_string(), Box::new(BFLNode::Number(1))),
        BFLNode::If(
            Box::new(BFLNode::Variable("x".to_string())),
            vec![
                BFLNode::Assign("y".to_string(), Box::new(BFLNode::Number(42))),
            ],
        ),
    ]);
    compiler.compile(&program).unwrap();
    let bf_code = compiler.get_output();
    println!("If test BF code length: {}", bf_code.len());
    println!("First 2000 chars of BF code: {}", &bf_code[..bf_code.len().min(2000)]);
    let mut bf = BF::new(bf_code, Mode::BFA);
    bf.run().unwrap();
    // Check that y == 42
    let x_addr = compiler.get_variable_address("x").unwrap();
    let y_addr = compiler.get_variable_address("y").unwrap();
    println!("x address: {}, x value: {}", x_addr, bf.dump_cells(x_addr + 1)[x_addr]);
    println!("y address: {}, y value: {}", y_addr, bf.dump_cells(y_addr + 1)[y_addr]);
    assert_eq!(bf.dump_cells(y_addr + 1)[y_addr], 42);
}

#[test]
fn test_bfl_loop() {
    let mut compiler = BFLCompiler::new();
    let program = BFLNode::Block(vec![
        BFLNode::Assign("x".to_string(), Box::new(BFLNode::Number(0))),
        BFLNode::Assign("i".to_string(), Box::new(BFLNode::Number(5))),
        BFLNode::While(
            Box::new(BFLNode::Variable("i".to_string())),
            vec![
                BFLNode::Assign(
                    "x".to_string(),
                    Box::new(BFLNode::Add(
                        Box::new(BFLNode::Variable("x".to_string())),
                        Box::new(BFLNode::Number(1)),
                    )),
                ),
                BFLNode::Assign(
                    "i".to_string(),
                    Box::new(BFLNode::Sub(
                        Box::new(BFLNode::Variable("i".to_string())),
                        Box::new(BFLNode::Number(1)),
                    )),
                ),
            ],
        ),
    ]);
    compiler.compile(&program).unwrap();
    let bf_code = compiler.get_output();
    let mut bf = BF::new(bf_code, Mode::BFA);
    bf.run().unwrap();
    let x_addr = compiler.get_variable_address("x").unwrap();
    let i_addr = compiler.get_variable_address("i").unwrap();
    println!("x address: {}, x value: {}", x_addr, bf.dump_cells(x_addr + 1)[x_addr]);
    println!("i address: {}, i value: {}", i_addr, bf.dump_cells(i_addr + 1)[i_addr]);
    assert_eq!(bf.dump_cells(x_addr + 1)[x_addr], 5);
}

#[test]
fn test_bfl_printf() {
    let mut compiler = BFLCompiler::new();
    let program = BFLNode::Block(vec![
        BFLNode::Assign("msg".to_string(), Box::new(BFLNode::String("Hello, BFL!\n".to_string()))),
        BFLNode::Syscall(
            Box::new(BFLNode::Number(1)),
            vec![
                BFLNode::Number(1),
                BFLNode::Variable("msg".to_string()),
                BFLNode::Number(12),
            ],
        ),
    ]);
    compiler.compile(&program).unwrap();
    let bf_code = compiler.get_output();
    let mut bf = BF::new(bf_code, Mode::BFA);
    bf.run().unwrap();
    // No assert: visually check output for "Hello, BFL!"
}

#[test]
fn test_bfl_add() {
    // Test Add operation
    let mut compiler = BFLCompiler::new();
    let program = BFLNode::Block(vec![
        BFLNode::Assign("result".to_string(), Box::new(BFLNode::Add(
            Box::new(BFLNode::Number(3)),
            Box::new(BFLNode::Number(4)),
        ))),
    ]);
    compiler.compile(&program).unwrap();
    let bf_code = compiler.get_output();
    let mut bf = BF::new(bf_code, Mode::BFA);
    bf.run().unwrap();
    let result_addr = compiler.get_variable_address("result").unwrap();
    assert_eq!(bf.dump_cells(result_addr + 1)[result_addr], 7);
}

#[test]
fn test_bfl_sub() {
    // Test Sub operation
    let mut compiler = BFLCompiler::new();
    let program = BFLNode::Block(vec![
        BFLNode::Assign("result".to_string(), Box::new(BFLNode::Sub(
            Box::new(BFLNode::Number(10)),
            Box::new(BFLNode::Number(3)),
        ))),
    ]);
    compiler.compile(&program).unwrap();
    let bf_code = compiler.get_output();
    let mut bf = BF::new(bf_code, Mode::BFA);
    bf.run().unwrap();
    let result_addr = compiler.get_variable_address("result").unwrap();
    assert_eq!(bf.dump_cells(result_addr + 1)[result_addr], 7);
}

#[test]
fn test_bfl_simple_assignment() {
    // Test basic assignment without any control flow
    let mut compiler = BFLCompiler::new();
    let program = BFLNode::Block(vec![
        BFLNode::Assign("simple_var".to_string(), Box::new(BFLNode::Number(456))),
    ]);
    compiler.compile(&program).unwrap();
    let bf_code = compiler.get_output();
    println!("Simple assignment BF code length: {}", bf_code.len());
    let mut bf = BF::new(bf_code, Mode::BFA);
    bf.run().unwrap();
    let simple_addr = compiler.get_variable_address("simple_var").unwrap();
    println!("simple_var address: {}", simple_addr);
    println!("simple_var value: {}", bf.dump_cells(simple_addr + 1)[simple_addr]);
    assert_eq!(bf.dump_cells(simple_addr + 1)[simple_addr], 456);
}

#[test]
fn test_bfl_minimal_if() {
    // Minimal test: just assign a value inside an if block
    let mut compiler = BFLCompiler::new();
    let program = BFLNode::Block(vec![
        BFLNode::If(
            Box::new(BFLNode::Number(1)), // Always true
            vec![
                BFLNode::Assign("test_var".to_string(), Box::new(BFLNode::Number(123))),
            ],
        ),
    ]);
    compiler.compile(&program).unwrap();
    let bf_code = compiler.get_output();
    println!("Generated BF code length: {}", bf_code.len());
    println!("First 1000 chars of BF code: {}", &bf_code[..bf_code.len().min(1000)]);
    let mut bf = BF::new(bf_code, Mode::BFA);
    bf.run().unwrap();
    let test_addr = compiler.get_variable_address("test_var").unwrap();
    println!("test_var address: {}", test_addr);
    println!("test_var value: {}", bf.dump_cells(test_addr + 1)[test_addr]);
    assert_eq!(bf.dump_cells(test_addr + 1)[test_addr], 123);
}

#[test]
#[ignore]
fn test_bfl_syscall_read() {
    // This test is ignored by default because it requires user input.
    let mut compiler = BFLCompiler::new();
    let program = BFLNode::Block(vec![
        BFLNode::Assign("buf".to_string(), Box::new(BFLNode::Bytes(vec![0; 8]))),
        BFLNode::Syscall(
            Box::new(BFLNode::Number(0)),
            vec![
                BFLNode::Number(0),
                BFLNode::Variable("buf".to_string()),
                BFLNode::Number(8),
            ],
        ),
        BFLNode::Syscall(
            Box::new(BFLNode::Number(1)),
            vec![
                BFLNode::Number(1),
                BFLNode::Variable("buf".to_string()),
                BFLNode::Number(8),
            ],
        ),
    ]);
    compiler.compile(&program).unwrap();
    let bf_code = compiler.get_output();
    let mut bf = BF::new(bf_code, Mode::BFA);
    bf.run().unwrap();
    // Manually check that input is echoed back
}

#[test]
#[ignore]
fn test_bfl_network_socket() {
    // This test is ignored by default because it requires network permissions.
    let mut compiler = BFLCompiler::new();
    let program = BFLNode::Block(vec![
        BFLNode::Syscall(
            Box::new(BFLNode::Number(41)),
            vec![
                BFLNode::Number(2),
                BFLNode::Number(1),
                BFLNode::Number(0),
            ],
        ),
        BFLNode::Assign("fd".to_string(), Box::new(BFLNode::Variable("_syscall_result".to_string()))),
        BFLNode::Syscall(
            Box::new(BFLNode::Number(3)),
            vec![BFLNode::Variable("fd".to_string())],
        ),
    ]);
    compiler.compile(&program).unwrap();
    let bf_code = compiler.get_output();
    let mut bf = BF::new(bf_code, Mode::BFA);
    let _ = bf.run();
    // No assert: just check that no panic occurs
} 