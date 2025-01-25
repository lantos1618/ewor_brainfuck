use ewor_brainfuck::bfl::BFLNode;

fn main() -> Result<(), String> {
    let program = BFLNode::Block(vec![
        BFLNode::Assign(
            "message".to_string(),
            BFLNode::String("Hello, World!".to_string()),
        ),
        BFLNode::Syscall(Syscall::Write {
            fd: 1,
            buf: BFLNode::Variable("message".to_string()),
        }),
    ]);

    let mut bfl = BFL::new();
    let compiled = bfl.compile(&program)?;

    let mut vm = BFVM::new();
    vm.run(&compiled)?;
    Ok(())
}
