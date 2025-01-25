use ewor_brainfuck::{
    bfl::{BFLCompiler, BFLNode},
    bfvm::BFVM,
    Syscall,
};

fn main() -> anyhow::Result<()> {
    let program = BFLNode::Block(vec![
        BFLNode::Assign(
            "message".to_string(),
            Box::new(BFLNode::String("Hello, World!\n".to_string())),
        ),
        BFLNode::Syscall(Syscall::Write {
            fd: 1,
            buf: b"Hello, World!\n".to_vec(),
        }),
    ]);

    let mut compiler = BFLCompiler::new();
    let compiled = compiler.compile(&program)?;

    let mut vm = BFVM::new(1024); // 1KB memory
    vm.run(&compiled)?;

    Ok(())
}
