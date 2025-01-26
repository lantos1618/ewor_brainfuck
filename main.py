class Node:
    def __init__(self):
        self.ptr = 0  # Track current pointer position

class Value(Node):
    def __init__(self, value):
        super().__init__()
        self.value = value

class Var(Node):
    def __init__(self, name):
        super().__init__()
        self.name = name

class Assign(Node):
    def __init__(self, var, value):
        super().__init__()
        self.var = var
        self.value = value

class Add(Node):
    def __init__(self, left, right):
        super().__init__()
        self.left = left
        self.right = right

class Sub(Node):
    def __init__(self, left, right):
        super().__init__()
        self.left = left
        self.right = right

class While(Node):
    def __init__(self, condition, body):
        super().__init__()
        self.condition = condition
        self.body = body

class Syscall(Node):
    def __init__(self, num, args):
        super().__init__()
        self.num = num
        self.args = args
        self.result_cell = 8  # Syscall results are stored in cell 8

class String(Node):
    def __init__(self, value):
        super().__init__()
        self.value = value
        self.length = len(value)

class CodeGenerator:
    def __init__(self, debug=False):
        self.code = []
        self.vars = {}  # Map variable names to cell positions
        self.next_var = 10  # Start variables at cell 10
        self.ptr = 0  # Current pointer position
        self.debug = debug
        self.indent_level = 0
        self.allocations = {}  # Track memory allocations: {var_name: (start_pos, length)}
        
    def emit(self, code, comment=None):
        """Emit code with optional debug comment"""
        indent = "    " * self.indent_level
        if self.debug and comment:
            self.code.append(f"\n{indent}// {comment}\n")
        if self.debug:
            self.code.append(f"{indent}{code}")
        else:
            self.code.append(code)

    def move(self, target):
        """Generate code to move pointer to target cell"""
        if target > self.ptr:
            self.emit('>' * (target - self.ptr), f"Moving from cell {self.ptr} to cell {target}")
        elif target < self.ptr:
            self.emit('<' * (self.ptr - target), f"Moving from cell {self.ptr} to cell {target}")
        self.ptr = target

    def set_cell(self, value):
        """Set current cell to value"""
        self.emit('[-]', f"Clear cell {self.ptr}")
        if value > 0:
            self.emit('+' * value, f"Set cell {self.ptr} to {value}")

    def generate(self, node):
        if isinstance(node, Value):
            return self.gen_value(node)
        elif isinstance(node, Var):
            return self.gen_var(node)
        elif isinstance(node, Assign):
            return self.gen_assign(node)
        elif isinstance(node, Add):
            return self.gen_add(node)
        elif isinstance(node, Syscall):
            return self.gen_syscall(node)
        elif isinstance(node, String):
            return self.gen_string(node)
        # Add other node types as needed

    def gen_value(self, node):
        """Generate code for a literal value"""
        self.set_cell(node.value)
        return self.ptr

    def allocate_memory(self, length, var_name=None):
        """Allocate contiguous memory cells"""
        # Find a free block of memory
        pos = self.next_var
        while any(start <= pos < start + size 
                 for (start, size) in self.allocations.values()):
            pos += 1
            
        if var_name:
            self.allocations[var_name] = (pos, length)
            if self.debug:
                self.emit('', f"Allocated {length} cells for {var_name} at position {pos}")
                
        self.next_var = max(self.next_var, pos + length)
        return pos

    def gen_var(self, node):
        """Get or allocate cell for variable"""
        if node.name not in self.vars:
            pos = self.allocate_memory(1, node.name)
            self.vars[node.name] = pos
            if self.debug:
                self.emit('', f"Allocating variable {node.name} to cell {pos}")
        return self.vars[node.name]

    def gen_assign(self, node):
        """Generate code for assignment"""
        if self.debug:
            self.emit('', f"Assignment: {node.var.name}")
            self.indent_level += 1
            
        var_pos = self.gen_var(node.var)
        
        if isinstance(node.value, Value):
            self.move(var_pos)
            self.gen_value(node.value)
        elif isinstance(node.value, String):
            # Store string character by character
            string_pos = self.gen_string(node.value)
            if string_pos != var_pos:
                # If string was stored at a different position than the variable,
                # we need to update the variable to point to the string
                self.move(var_pos)
                self.set_cell(string_pos)
        elif isinstance(node.value, Syscall):
            # Generate syscall code
            self.gen_syscall(node.value)
            # Copy result from cell 8 to variable's cell
            self.move(8)  # Move to result cell
            self.emit('[-', f"Start copying result from cell 8 to cell {var_pos}")
            self.move(var_pos)
            self.emit('+', f"Copy value to cell {var_pos}")
            self.move(8)
            self.emit(']', f"End copying result")
            
        if self.debug:
            self.indent_level -= 1

    def gen_syscall(self, node):
        """Generate code for syscall"""
        if self.debug:
            self.emit('', f"Syscall {node.num} with {len(node.args)} args")
            self.indent_level += 1
        
        # Set syscall number in cell 1
        self.move(1)
        self.set_cell(node.num)
        
        # Set arguments in cells 2-7
        for i, arg in enumerate(node.args):
            if self.debug:
                self.emit('', f"Setting syscall arg {i}")
                self.indent_level += 1
            self.move(2 + i)
            if isinstance(arg, Value):
                self.gen_value(arg)
            elif isinstance(arg, Var):
                # Copy from variable cell to argument cell
                var_pos = self.gen_var(arg)
                # First clear the target cell
                self.move(2 + i)
                self.set_cell(0)
                # Move to source and start copy loop
                self.move(var_pos)
                self.emit('[-', f"Start copying from cell {var_pos} to cell {2 + i}")
                self.move(2 + i)
                self.emit('+', f"Copy to argument cell")
                self.move(var_pos)
                self.emit(']', f"End copying")
            if self.debug:
                self.indent_level -= 1
        
        # Trigger syscall with cell 0
        self.move(0)
        self.set_cell(255)
        self.emit('.', "Trigger syscall")
        
        if self.debug:
            self.indent_level -= 1

    def gen_add(self, node):
        """Generate code for addition"""
        # TODO: Implement addition
        pass

    def gen_string(self, node):
        """Generate code for string storage"""
        # Allocate space for the string
        start_pos = self.allocate_memory(node.length)
        
        if self.debug:
            self.emit('', f"Storing string '{node.value}' starting at cell {start_pos}")
            self.indent_level += 1
            
        # Store each character
        for i, char in enumerate(node.value):
            self.move(start_pos + i)
            self.set_cell(ord(char))
            if self.debug:
                self.emit('', f"Stored '{char}' (ASCII {ord(char)}) in cell {start_pos + i}")
            
        if self.debug:
            self.indent_level -= 1
            
        return start_pos  # Return starting position of string

class BrainfuckVM:
    def __init__(self, code, memory_size=30000, debug=False, debug_cells=16):
        self.code = code
        self.memory = [0] * memory_size
        self.data_ptr = 0
        self.code_ptr = 0
        self.debug = debug
        self.debug_cells = debug_cells
        self.bracket_map = self._build_bracket_map()

    def _build_bracket_map(self):
        """Build map of matching brackets for faster loop execution"""
        stack = []
        bracket_map = {}
        
        for pos, cmd in enumerate(self.code):
            if cmd == '[':
                stack.append(pos)
            elif cmd == ']':
                if stack:
                    start = stack.pop()
                    bracket_map[start] = pos
                    bracket_map[pos] = start
        
        return bracket_map

    def _debug_memory(self):
        """Print debug view of memory"""
        if not self.debug:
            return
            
        cells = self.memory[:self.debug_cells]
        
        # Create a visual table
        width = 16  # cells per row
        cell_width = 4
        border_h = "─"
        border_v = "│"
        corner_t = "┬"
        corner_b = "┴"
        corner_l = "├"
        corner_r = "┤"
        corner_tl = "┌"
        corner_tr = "┐"
        corner_bl = "└"
        corner_br = "┘"
        corner = "┼"
        
        def make_row(prefix, values, formatter):
            row = [prefix.ljust(8) + border_v]
            for i, v in enumerate(values):
                row.append(formatter(v).center(cell_width))
                if i < len(values) - 1:  # Add vertical border between cells
                    row.append(border_v)
            row.append(border_v)  # End border
            return "".join(row)
        
        def make_border(left_corner, mid_corner, right_corner):
            parts = [" " * 8 + left_corner]
            for i in range(width):
                parts.append(border_h * cell_width)
                if i < width - 1:
                    parts.append(mid_corner)
            parts.append(right_corner)
            return "".join(parts)
            
        # Format cells
        idx_row = make_row("Cell", range(len(cells)), lambda x: str(x))
        val_row = make_row("Value", cells, lambda x: str(x))
        char_row = make_row("Char", cells, lambda x: chr(x) if 32 <= x <= 126 else ".")
        ptr_row = make_row("Pointer", range(len(cells)), 
                          lambda x: "^" if x == self.data_ptr else " ")
        
        # Print the table
        print("\nMemory Dump:")
        print(make_border(corner_tl, corner_t, corner_tr))
        print(idx_row)
        print(make_border(corner_l, corner, corner_r))
        print(val_row)
        print(make_border(corner_l, corner, corner_r))
        print(char_row)
        print(make_border(corner_l, corner, corner_r))
        print(ptr_row)
        print(make_border(corner_bl, corner_b, corner_br))
        print()

    def _handle_syscall(self):
        """Handle syscall when cell 0 is 255 and '.' is executed"""
        import ctypes
        import platform
        
        syscall_num = self.memory[1]
        args = self.memory[2:8]  # Get up to 6 arguments
        
        if self.debug:
            print(f"\nSyscall: {syscall_num} with args {args[:3]}")  # Show first 3 args
        
        # Filter out zero args from the end
        while args and args[-1] == 0:
            args.pop()

        try:
            # Get libc
            if platform.system() == 'Darwin':
                libc = ctypes.CDLL('libc.dylib')
            else:
                libc = ctypes.CDLL('libc.so.6')

            # For write syscall, we need to create a buffer from our memory
            if syscall_num == 1:  # write
                fd = args[0]
                buf_start = args[1]
                count = args[2]
                
                # Create buffer from our memory
                buf = bytes(self.memory[buf_start:buf_start + count])
                if self.debug:
                    print(f"Write: fd={fd}, buf=[{buf.decode('utf-8')}], len={count}")
                
                result = libc.write(fd, buf, count)
                if self.debug:
                    print(f"Result: {result}")
            else:
                # For other syscalls, pass args directly
                result = libc.syscall(syscall_num, *args)
                if self.debug:
                    print(f"Result: {result}")
                
            self.memory[8] = result & 0xFF  # Store result in cell 8
        except Exception as e:
            print(f"Syscall error: {e}")
            self.memory[8] = 0xFF  # Error indicator

    def run(self):
        while self.code_ptr < len(self.code):
            cmd = self.code[self.code_ptr]
            
            if cmd == '>':
                self.data_ptr = (self.data_ptr + 1) % len(self.memory)
            elif cmd == '<':
                self.data_ptr = (self.data_ptr - 1) % len(self.memory)
            elif cmd == '+':
                self.memory[self.data_ptr] = (self.memory[self.data_ptr] + 1) % 256
            elif cmd == '-':
                self.memory[self.data_ptr] = (self.memory[self.data_ptr] - 1) % 256
            elif cmd == '.':
                # Check for syscall
                if self.data_ptr == 0 and self.memory[self.data_ptr] == 255:
                    if self.debug:
                        print("\n=== Syscall Triggered ===")
                        self._debug_memory()
                    self._handle_syscall()
                    if self.debug:
                        print("=== Syscall Complete ===")
                        self._debug_memory()
                else:
                    # Normal output
                    print(chr(self.memory[self.data_ptr]), end='')
            elif cmd == ',':
                # Read a single byte
                try:
                    self.memory[self.data_ptr] = ord(input()[0])
                except:
                    self.memory[self.data_ptr] = 0
            elif cmd == '[':
                if self.memory[self.data_ptr] == 0:
                    self.code_ptr = self.bracket_map[self.code_ptr]
            elif cmd == ']':
                if self.memory[self.data_ptr] != 0:
                    self.code_ptr = self.bracket_map[self.code_ptr]
            
            self.code_ptr += 1

# Update the example to use debug mode
if __name__ == "__main__":
    # Test memory allocation with multiple strings
    program = [
        # Store first string
        Assign(Var('message'), String("hi")),
        # Store second string
        Assign(Var('message2'), String("\nhello world!\n")),
        # write first string
        Assign(Var('result'), 
            Syscall(1, [Value(1), Var('message'), Value(2)])
        ),
        # write second string
        Assign(Var('result2'), 
            Syscall(1, [Value(1), Var('message2'), Value(15)])
        )
    ]

    gen = CodeGenerator(debug=True)
    for node in program:
        gen.generate(node)
    print('Generated Brainfuck code:')
    bf_code = ''.join(gen.code)
    print(bf_code)
    
    print('\nRunning code in VM:')
    vm = BrainfuckVM(bf_code, debug=True)
    vm.run()
