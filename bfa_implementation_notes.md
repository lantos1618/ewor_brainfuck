# BFA (BrainFuck Assembler) Implementation Notes

## Architecture Support
### ARM64 (macOS M1/M2)
```
x8  - syscall number
x0  - arg1
x1  - arg2
x2  - arg3
x3  - arg4
x4  - arg5
x5  - arg6

Syscalls:
socket   97
bind     104
listen   106
accept   98
connect  98
send     101
recv     102
close    6
```

### x86_64
```
rax - syscall number
rdi - arg1
rsi - arg2
rdx - arg3
r10 - arg4
r8  - arg5
r9  - arg6

Syscalls:
socket   41
bind     49
listen   50
accept   43
connect  42
send     44
recv     45
close    3
```

## BFA Memory Layout
```
[syscall_flag | syscall_num | args... | program_memory]
     cell 0    |   cell 1   | 2-7  |     8+
```

## Extended Instructions
- All standard BF ops remain same
- When cell[0] = 1, next '.' triggers syscall
- Args must be set before syscall

## Implementation Steps
1. Base interpreter
   - Standard BF ops
   - Memory management
   - Instruction parsing

2. Syscall Extension
   - Arch detection
   - Syscall table
   - Register mapping

3. Socket Implementation
   - Helper BF functions for common socket ops
   - Error handling patterns
   - Buffer management

4. Testing Plan
   - Unit tests per syscall
   - Socket communication tests
   - Full chat protocol test 