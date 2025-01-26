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

    def gen_var(self, node):
        """Get or allocate cell for variable"""
        if node.name not in self.vars:
            self.vars[node.name] = self.next_var
            if self.debug:
                self.emit('', f"Allocating variable {node.name} to cell {self.next_var}")
            self.next_var += 1
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
                self.move(var_pos)
                # TODO: Add code to copy value
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
        start_pos = self.next_var
        self.next_var += node.length  # Reserve cells for each character
        
        if self.debug:
            self.emit('', f"Storing string '{node.value}' starting at cell {start_pos}")
            self.indent_level += 1
            
        # Store each character
        for i, char in enumerate(node.value):
            self.move(start_pos + i)
            self.set_cell(ord(char))
            
        if self.debug:
            self.indent_level -= 1
            
        return start_pos  # Return starting position of string

class BrainfuckVM:
    def __init__(self, code, memory_size=30000):
        self.code = code
        self.memory = [0] * memory_size
        self.data_ptr = 0
        self.code_ptr = 0
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

    def _handle_syscall(self):
        """Handle syscall when cell 0 is 255 and '.' is executed"""
        syscall_num = self.memory[1]
        args = self.memory[2:8]  # Get up to 6 arguments
        
        # Filter out zero args from the end
        while args and args[-1] == 0:
            args.pop()
            
        try:
            import os
            result = os.syscall(syscall_num, *args)
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
                    self._handle_syscall()
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
    # Write "Hello, World\n" to stdout (fd 1)
    program = [
        # Store the string
        Assign(Var('message'), String("Hello, World\n")),
        # write(1, message, 13)
        Assign(Var('result'), 
            Syscall(1, [  # syscall 1 is write
                Value(1),  # fd 1 is stdout
                Var('message'),  # pointer to string
                Value(13)  # length of string
            ])
        )
    ]

    gen = CodeGenerator(debug=True)
    for node in program:
        gen.generate(node)
    print('Generated Brainfuck code:')
    bf_code = ''.join(gen.code)
    print(bf_code)
    
    print('\nRunning code in VM:')
    vm = BrainfuckVM(bf_code)
    vm.run()
