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

class Eq(Node):
    def __init__(self, left, right):
        super().__init__()
        self.left = left
        self.right = right

class Lt(Node):
    def __init__(self, left, right):
        super().__init__()
        self.left = left
        self.right = right

class Lte(Node):
    def __init__(self, left, right):
        super().__init__()
        self.left = left
        self.right = right

class Gt(Node):
    def __init__(self, left, right):
        super().__init__()
        self.left = left
        self.right = right

class Gte(Node):
    def __init__(self, left, right):
        super().__init__()
        self.left = left
        self.right = right

class If(Node):
    def __init__(self, condition, then_branch, else_branch=None):
        super().__init__()
        self.condition = condition
        self.then_branch = then_branch
        self.else_branch = else_branch

class Loop(Node):
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
        self.output = []
        self.vars = {}  # Maps variable names to their allocated cell
        self.next_var = 10
        self.ptr = 0
        self.debug = debug
        self.indent_level = 0
        self.allocations = []  # Tracks all allocated blocks as (start, size)

        
    def emit(self, code, comment=None):
        """Emit code with optional debug comment"""
        indent = "    " * self.indent_level
        debug_info = f"{indent}// {comment}" if self.debug and comment else None
        code_str = f"{indent}{code}" if self.debug and code else code
        self.output.append((code_str if code else None, debug_info))

    def get_code(self):
        """Get just the Brainfuck code without debug info"""
        return ''.join(code for code, _ in self.output if code is not None)

    def move_ptr(self, target):
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
        elif isinstance(node, Sub):
            return self.gen_sub(node)
        elif isinstance(node, Syscall):
            return self.gen_syscall(node)
        elif isinstance(node, String):
            return self.gen_string(node)
        elif isinstance(node, Eq):
            return self.gen_eq(node)
        elif isinstance(node, Lt):
            return self.gen_lt(node)
        elif isinstance(node, Lte):
            return self.gen_lte(node)
        elif isinstance(node, Gt):
            return self.gen_gt(node)
        elif isinstance(node, Gte):
            return self.gen_gte(node)
        elif isinstance(node, If):
            return self.gen_if(node)
        elif isinstance(node, Loop):
            return self.gen_loop(node)
        # Add other node types as needed

    def gen_value(self, node):
        """Generate code for a literal value"""
        self.set_cell(node.value)
        return self.ptr

    def allocate_memory(self, length, var_name=None):
        """Allocate contiguous memory cells, ensuring no overlap with existing allocations."""
        pos = self.next_var
        while True:
            overlap = False
            for (start, size) in self.allocations:
                # Check if current pos overlaps with any existing block
                if (start <= pos < start + size) or (pos <= start < pos + length):
                    overlap = True
                    break
            if not overlap:
                break
            pos += 1

        self.allocations.append((pos, length))
        if var_name:
            self.vars[var_name] = pos  # Variables are single-cell
            if self.debug:
                self.emit('', f"Allocated variable {var_name} at cell {pos}")

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
            self.move_ptr(var_pos)
            self.gen_value(node.value)
        elif isinstance(node.value, String):
            # Store string character by character
            string_pos = self.gen_string(node.value)
            if string_pos != var_pos:
                # If string was stored at a different position than the variable,
                # we need to update the variable to point to the string
                self.move_ptr(var_pos)
                self.set_cell(string_pos)
        elif isinstance(node.value, Syscall):
            # Generate syscall code
            self.gen_syscall(node.value)
            # Move result from cell 8 to variable's cell
            self.move_cell(8, var_pos)
            
        if self.debug:
            self.indent_level -= 1

    def gen_syscall(self, node):
        """Generate code for syscall"""
        if self.debug:
            self.emit('', f"Syscall {node.num} with {len(node.args)} args")
            self.indent_level += 1
        
        # Set syscall number in cell 1
        self.move_ptr(1)
        self.set_cell(node.num)
        
        # Set arguments in cells 2-7
        for i, arg in enumerate(node.args):
            if self.debug:
                self.emit('', f"Setting syscall arg {i}")
                self.indent_level += 1
            self.move_ptr(2 + i)
            if isinstance(arg, Value):
                self.gen_value(arg)
            elif isinstance(arg, Var):
                # Copy from variable cell to argument cell, preserving the variable
                var_pos = self.gen_var(arg)
                self.copy_cell(var_pos, 2 + i, temp=9)  # Use cell 9 as temp
            if self.debug:
                self.indent_level -= 1
        
        # Trigger syscall with cell 0
        self.move_ptr(0)
        self.set_cell(255)
        self.emit('.', "Trigger syscall")
        
        if self.debug:
            self.indent_level -= 1

    def gen_add(self, node):
        """Generate code for addition"""
        if self.debug:
            self.emit('', f"Addition operation")
            self.indent_level += 1
            
        # Generate code for left operand
        left_pos = None
        if isinstance(node.left, Value):
            temp_pos = self.allocate_memory(1)
            self.move_ptr(temp_pos)
            self.gen_value(node.left)
            left_pos = temp_pos
        elif isinstance(node.left, Var):
            left_pos = self.gen_var(node.left)
        
        # Generate code for right operand
        right_pos = None
        if isinstance(node.right, Value):
            temp_pos = self.allocate_memory(1)
            self.move_ptr(temp_pos)
            self.gen_value(node.right)
            right_pos = temp_pos
        elif isinstance(node.right, Var):
            right_pos = self.gen_var(node.right)
        
        # Allocate result cell
        result_pos = self.allocate_memory(1)
        temp_pos = self.allocate_memory(1)  # For preserving right operand
        
        # Copy left operand to result
        self.copy_cell(left_pos, result_pos)
        
        # Add right operand to result
        self.copy_cell(right_pos, temp_pos)  # Preserve right operand
        self.move_ptr(temp_pos)
        self.emit('[-', f"Add loop start")
        self.move_ptr(result_pos)
        self.emit('+', f"Increment result")
        self.move_ptr(temp_pos)
        self.emit(']', f"Add loop end")
        
        if self.debug:
            self.indent_level -= 1
        
        return result_pos

    def gen_sub(self, node):
        """Generate code for subtraction"""
        if self.debug:
            self.emit('', f"Subtraction operation")
            self.indent_level += 1
            
        # Generate code for left operand
        left_pos = None
        if isinstance(node.left, Value):
            temp_pos = self.allocate_memory(1)
            self.move_ptr(temp_pos)
            self.gen_value(node.left)
            left_pos = temp_pos
        elif isinstance(node.left, Var):
            left_pos = self.gen_var(node.left)
        
        # Generate code for right operand
        right_pos = None
        if isinstance(node.right, Value):
            temp_pos = self.allocate_memory(1)
            self.move_ptr(temp_pos)
            self.gen_value(node.right)
            right_pos = temp_pos
        elif isinstance(node.right, Var):
            right_pos = self.gen_var(node.right)
        
        # Allocate result cell
        result_pos = self.allocate_memory(1)
        temp_pos = self.allocate_memory(1)  # For preserving right operand
        
        # Copy left operand to result
        self.copy_cell(left_pos, result_pos)
        
        # Subtract right operand from result
        self.copy_cell(right_pos, temp_pos)  # Preserve right operand
        self.move_ptr(temp_pos)
        self.emit('[-', f"Subtract loop start")
        self.move_ptr(result_pos)
        self.emit('-', f"Decrement result")
        self.move_ptr(temp_pos)
        self.emit(']', f"Subtract loop end")
        
        if self.debug:
            self.indent_level -= 1
        
        return result_pos

    def gen_string(self, node):
        """Generate code for string storage"""
        # Allocate space for the string
        start_pos = self.allocate_memory(node.length)
        
        if self.debug:
            self.emit('', f"Storing string '{node.value}' starting at cell {start_pos}")
            self.indent_level += 1
            
        # Store each character
        for i, char in enumerate(node.value):
            self.move_ptr(start_pos + i)
            self.set_cell(ord(char))
            if self.debug:
                self.emit('', f"Stored '{char}' (ASCII {ord(char)}) in cell {start_pos + i}")
            
        if self.debug:
            self.indent_level -= 1
            
        return start_pos  # Return starting position of string

    def copy_cell(self, source, dest, temp=None):
        """Copy value from source cell to dest cell, optionally using temp cell.
        If temp is not provided, source will be zeroed."""
        if self.debug:
            self.emit('', f"Copying from cell {source} to cell {dest}")
            self.indent_level += 1

        if temp is not None:
            # First clear temp and dest
            self.move_ptr(temp)
            self.set_cell(0)
            self.move_ptr(dest)
            self.set_cell(0)
            
            # Copy source to both temp and dest
            self.move_ptr(source)
            self.emit('[-', f"Start copying from cell {source}")
            self.move_ptr(dest)
            self.emit('+', f"Copy to dest cell {dest}")
            self.move_ptr(temp)
            self.emit('+', f"Copy to temp cell {temp}")
            self.move_ptr(source)
            self.emit(']', f"End first copy phase")
            
            # Copy back from temp to source
            self.move_ptr(temp)
            self.emit('[-', f"Start restoring source from temp")
            self.move_ptr(source)
            self.emit('+', f"Restore to source cell {source}")
            self.move_ptr(temp)
            self.emit(']', f"End restore phase")
        else:
            # Direct copy that consumes source
            self.move_ptr(dest)
            self.set_cell(0)
            self.move_ptr(source)
            self.emit('[-', f"Start moving from cell {source}")
            self.move_ptr(dest)
            self.emit('+', f"Copy to cell {dest}")
            self.move_ptr(source)
            self.emit(']', f"End moving")

        if self.debug:
            self.indent_level -= 1

    def move_cell(self, source, dest):
        """Move value from source cell to dest cell (source will be zeroed)"""
        if self.debug:
            self.emit('', f"Moving from cell {source} to cell {dest}")
        self.copy_cell(source, dest)  # Since we're not providing temp, this will zero source

    def gen_eq(self, node):
        """Generate code for equality comparison"""
        if self.debug:
            self.emit('', f"Equality comparison")
            self.indent_level += 1
            
        # Generate code for operands
        left_pos = self.generate(node.left)
        right_pos = self.generate(node.right)
        
        # Allocate cells for result and temporary values
        result_pos = self.allocate_memory(1)
        temp1_pos = self.allocate_memory(1)
        temp2_pos = self.allocate_memory(1)
        
        # Copy operands to temp cells
        self.copy_cell(left_pos, temp1_pos)
        self.copy_cell(right_pos, temp2_pos)
        
        # Set result to 1 initially
        self.move_ptr(result_pos)
        self.set_cell(1)
        
        # Subtract operands and check if both reach zero simultaneously
        self.move_ptr(temp1_pos)
        self.emit('[-', "Start comparison loop")
        self.move_ptr(temp2_pos)
        self.emit('-', "Decrement second operand")
        self.move_ptr(result_pos)
        self.emit('-', "Clear result if mismatch")
        self.move_ptr(temp1_pos)
        self.emit(']', "End comparison loop")
        
        # Check if second operand has remaining value
        self.move_ptr(temp2_pos)
        self.emit('[', "Check remaining value")
        self.move_ptr(result_pos)
        self.emit('[-]', "Clear result if second operand has remaining value")
        self.move_ptr(temp2_pos)
        self.emit('[-]', "Clear remaining value")
        self.emit(']', "End check")
        
        if self.debug:
            self.indent_level -= 1
            
        return result_pos

    def gen_lt(self, node):
        """Generate code for less than comparison"""
        if self.debug:
            self.emit('', f"Less than comparison")
            self.indent_level += 1
            
        # Generate code for operands
        left_pos = self.generate(node.left)
        right_pos = self.generate(node.right)
        
        # Allocate cells for result and temporary values
        result_pos = self.allocate_memory(1)
        temp1_pos = self.allocate_memory(1)
        temp2_pos = self.allocate_memory(1)
        
        # Copy operands to temp cells
        self.copy_cell(left_pos, temp1_pos)
        self.copy_cell(right_pos, temp2_pos)
        
        # Initialize result to 0
        self.move_ptr(result_pos)
        self.set_cell(0)
        
        # Subtract both numbers simultaneously until one reaches zero
        self.move_ptr(temp1_pos)
        self.emit('[', "Start comparison loop")
        self.emit('-', "Decrement first operand")
        self.move_ptr(temp2_pos)
        self.emit('-', "Decrement second operand")
        self.emit('[', "If second operand still has value")
        self.move_ptr(result_pos)
        self.emit('+', "Set result to 1")
        self.move_ptr(temp2_pos)
        self.emit('[-]', "Clear second operand")
        self.emit(']', "End second check")
        self.move_ptr(temp1_pos)
        self.emit(']', "End comparison loop")
        
        if self.debug:
            self.indent_level -= 1
            
        return result_pos

    def gen_lte(self, node):
        """Generate code for less than or equal comparison"""
        if self.debug:
            self.emit('', f"Less than or equal comparison")
            self.indent_level += 1
            
        # We can implement this as (a <= b) ≡ !(b < a)
        gt_node = Lt(node.right, node.left)  # Swap operands for greater than
        gt_pos = self.gen_lt(gt_node)
        
        # Allocate result cell
        result_pos = self.allocate_memory(1)
        
        # Invert the result (1 - gt_result)
        self.move_ptr(result_pos)
        self.set_cell(1)
        self.move_ptr(gt_pos)
        self.emit('[-', "Start inversion")
        self.move_ptr(result_pos)
        self.emit('-', "Subtract from 1")
        self.move_ptr(gt_pos)
        self.emit(']', "End inversion")
        
        if self.debug:
            self.indent_level -= 1
            
        return result_pos

    def gen_gt(self, node):
        """Generate code for greater than comparison"""
        if self.debug:
            self.emit('', f"Greater than comparison")
            self.indent_level += 1
            
        # We can implement this as (a > b) ≡ (b < a)
        lt_node = Lt(node.right, node.left)  # Swap operands
        result_pos = self.gen_lt(lt_node)
        
        if self.debug:
            self.indent_level -= 1
            
        return result_pos

    def gen_gte(self, node):
        """Generate code for greater than or equal comparison"""
        if self.debug:
            self.emit('', f"Greater than or equal comparison")
            self.indent_level += 1
            
        # We can implement this as (a >= b) ≡ !(a < b)
        lt_pos = self.gen_lt(node)
        
        # Allocate result cell
        result_pos = self.allocate_memory(1)
        
        # Invert the result (1 - lt_result)
        self.move_ptr(result_pos)
        self.set_cell(1)
        self.move_ptr(lt_pos)
        self.emit('[-', "Start inversion")
        self.move_ptr(result_pos)
        self.emit('-', "Subtract from 1")
        self.move_ptr(lt_pos)
        self.emit(']', "End inversion")
        
        if self.debug:
            self.indent_level -= 1
            
        return result_pos

    def gen_if(self, node):
        """Generate code for if statement"""
        if self.debug:
            self.emit('', f"If statement")
            self.indent_level += 1
            
        # Generate code for condition
        cond_pos = self.generate(node.condition)
        
        # Allocate cells for then and else results
        result_pos = self.allocate_memory(1)
        temp_pos = self.allocate_memory(1)  # For preserving condition
        
        # Copy condition to temp (preserving it)
        self.copy_cell(cond_pos, temp_pos)
        
        # Start if block
        self.move_ptr(temp_pos)
        self.emit('[', "Start if block")
        
        # Generate and store then branch result
        then_result = self.generate(node.then_branch)
        if then_result is not None:
            self.copy_cell(then_result, result_pos)
            
        # Clear the condition to prevent else execution
        self.move_ptr(temp_pos)
        self.emit('[-]', "Clear condition")
        self.emit(']', "End if block")
        
        # Handle else branch if it exists
        if node.else_branch is not None:
            # Copy original condition to check if it was false
            self.copy_cell(cond_pos, temp_pos)
            
            # Invert condition for else (if cond was 0, make it 1)
            self.move_ptr(temp_pos)
            self.emit('[[-]', "If condition was true")
            self.emit(']', "End true check")
            self.emit('+', "Set to 1 if was false")
            
            # Start else block
            self.emit('[', "Start else block")
            
            # Generate and store else branch result
            else_result = self.generate(node.else_branch)
            if else_result is not None:
                self.copy_cell(else_result, result_pos)
                
            # Clear the inverted condition
            self.move_ptr(temp_pos)
            self.emit('[-]', "Clear inverted condition")
            self.emit(']', "End else block")
        
        if self.debug:
            self.indent_level -= 1
            
        return result_pos

    def gen_loop(self, node):
        """Generate code for loop"""
        if self.debug:
            self.emit('', f"Loop")
            self.indent_level += 1
            
        # Allocate cells for condition and body results
        result_pos = self.allocate_memory(1)
        cond_pos = self.allocate_memory(1)
        temp_pos = self.allocate_memory(1)  # For preserving condition
        
        # Generate initial condition
        initial_cond = self.generate(node.condition)
        self.copy_cell(initial_cond, cond_pos)
        
        # Start loop
        self.move_ptr(cond_pos)
        self.emit('[', "Start loop")
        
        # Generate and execute body
        body_result = self.generate(node.body)
        if body_result is not None:
            self.copy_cell(body_result, result_pos)
        
        # Generate next condition
        next_cond = self.generate(node.condition)
        self.copy_cell(next_cond, cond_pos)
        
        # End loop
        self.move_ptr(cond_pos)
        self.emit(']', "End loop")
        
        if self.debug:
            self.indent_level -= 1
            
        return result_pos

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
                    print(f"Syscall Result: {result}")
            else:
                # For other syscalls, pass args directly
                result = libc.syscall(syscall_num, *args)
                if self.debug:
                    print(f"Syscall Result: {result}")
                
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
    # Test control flow operations
    program = [
        # Initialize variables
        Assign(Var('x'), Value(5)),
        Assign(Var('y'), Value(3)),
        
        # Test if-else with comparison
        If(
            Lt(Var('y'), Var('x')),
            Assign(Var('message1'), String("First: x is greater than y\n")),
            Assign(Var('message1'), String("First: x is not greater than y\n"))
        ),
        
        # Print first message
        Syscall(1, [Value(1), Var('message1'), Value(31)]),
        
        # Test loop - count down y to 0
        Loop(
            Var('y'),  # Continue while y > 0
            Assign(Var('y'), Sub(Var('y'), Value(1)))
        ),
        
        # Test if after loop
        If(
            Eq(Var('y'), Value(0)),
            Assign(Var('message2'), String("Second: y is now zero\n")),
            Assign(Var('message2'), String("Second: y is not zero\n"))
        ),
        
        # Print second message
        Syscall(1, [Value(1), Var('message2'), Value(30)])
    ]

    gen = CodeGenerator(debug=True)
    for node in program:
        gen.generate(node)
    
    if gen.debug:
        print('Generated Brainfuck code with debug info:')
        for code, debug in gen.output:
            if debug:
                print(debug)
            if code:
                print(code)
    else:
        print('Generated Brainfuck code:')
        print(gen.get_code())
    
    print('\nRunning code in BFVM:')
    # Run the actual Brainfuck code without debug comments
    vm = BrainfuckVM(gen.get_code(), debug=True)
    vm.run()
