# ewor_brainfuck Architecture

## Overview

**ewor_brainfuck** is a system for compiling a higher-level language (BFL) into Brainfuck, with extensions for system calls (BFA). It enables complex operations, such as networking, within the constraints of Brainfuck by introducing a syscall interface and a structured memory model.

---

## Components

### 1. BFL (Brainfuck-Like Language)
- **Purpose:** A higher-level language with variables, arithmetic, control flow, and syscalls.
- **AST:** Represented by `BFLNode` enum in `bfl.rs`.
- **Features:**
  - Variables and assignment
  - Numbers, strings, and byte arrays
  - Arithmetic: Add, Sub
  - Control flow: If, While
  - Syscalls: Socket, bind, listen, accept, read, write, close, etc.

### 2. BFL Compiler (`bfl.rs`)
- **Role:** Compiles BFL AST into Brainfuck code with syscall extensions (BFA).
- **Variable Model:**
  - Variables are mapped to fixed memory cells.
  - Strings/bytes are stored in high memory, with variables holding pointers.
  - Non-destructive value copying is used for assignments and syscall argument passing.
- **Syscall Convention:**
  - Syscall number in `cell[7]`
  - Arguments in `cell[1]` through `cell[6]`
  - Return value in `cell[0]` (also accessible as `_syscall_result`)
- **Pointer Tracking:**
  - The compiler tracks the current pointer position and emits `>`/`<` as needed.

### 3. Brainfuck Interpreter (`bf.rs`)
- **Role:** Executes Brainfuck or BFA code.
- **Modes:**
  - `Mode::BF`: Standard Brainfuck
  - `Mode::BFA`: Brainfuck with syscall extensions
- **Syscall Handling:**
  - On `.` in BFA mode, reads syscall number and arguments from memory, performs the syscall, and stores the result.
  - Strict memory map for syscall interface (see below).
- **Error Handling:**
  - Robust error types for memory, syscalls, and bracket mismatches.

---

## Memory Model

- **Cells 0-7:** Reserved for syscall interface
  - `cell[0]`: Return value (also `_syscall_result` in BFL)
  - `cell[1]`-`cell[6]`: Syscall arguments (RDI-R9)
  - `cell[7]`: Syscall number (RAX)
- **Cells 8+**: User variables, buffers, and data
- **High memory (e.g., 30000+):** Used for scratch space during value copying

---

## Syscall Convention

| Cell      | Purpose                |
|-----------|------------------------|
| 0         | Return value           |
| 1         | Arg 1 (fd, etc.)       |
| 2         | Arg 2 (buffer ptr)     |
| 3         | Arg 3 (length, etc.)   |
| 4         | Arg 4                  |
| 5         | Arg 5                  |
| 6         | Arg 6                  |
| 7         | Syscall number         |

- **Supported syscalls:** socket, bind, listen, accept, read, write, close (Linux x86_64 numbers)
- **Example:** To write to stdout: set `cell[7]=1`, `cell[1]=1`, `cell[2]=buffer_addr`, `cell[3]=length`, then execute `.`

---

## Example: BFL Echo Server

```rust
BFLNode::Block(vec![
    // socket(AF_INET, SOCK_STREAM, 0)
    BFLNode::Syscall(Box::new(BFLNode::Number(41)), vec![
        BFLNode::Number(2),
        BFLNode::Number(1),
        BFLNode::Number(0),
    ]),
    BFLNode::Assign("socket_fd".to_string(), Box::new(BFLNode::Variable("_syscall_result".to_string()))),
    // ...
])
```

---

## Design Strengths
- **Separation of Concerns:** Compiler and interpreter are distinct.
- **Extensible:** New syscalls and language features can be added.
- **Robust Error Handling:** Clear error types and propagation.
- **Testable:** Good test coverage for both interpreter and compiler.

---

## Contributing
- See `notes.md` for design notes and thought process.
- See `src/bfl.rs` and `src/bf.rs` for implementation details.
- For BFL syntax and usage, see this file and the test cases in `src/bfl.rs`. 