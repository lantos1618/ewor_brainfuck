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

class Block(Node):
    def __init__(self, statements):
        super().__init__()
        self.statements = statements

class If(Node):
    """
    Simple if statement: if condition != 0, execute then_block.
    else_block is ignored here for simplicity.
    """
    def __init__(self, condition, then_block, else_block=None):
        super().__init__()
        self.condition = condition
        self.then_block = then_block
        self.else_block = else_block  # not used below, but you could extend later.

class Eq(Node):
    """
    Equality check: result is 1 if left == right else 0
    We store that 0/1 in the cell used by 'generate(Eq)'.
    """
    def __init__(self, left, right):
        super().__init__()
        self.left = left
        self.right = right

class Lt(Node):
    """
    Less-than check: result is 1 if left < right else 0
    For an 8-bit toy approach, we do a naive subtract-based check.
    """
    def __init__(self, left, right):
        super().__init__()
        self.left = left
        self.right = right

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
        elif isinstance(node, Syscall):
            return self.gen_syscall(node)
        elif isinstance(node, String):
            return self.gen_string(node)
        elif isinstance(node, Block):
            return self.gen_block(node)
        elif isinstance(node, If):
            return self.gen_if(node)
        elif isinstance(node, Eq):
            return self.gen_eq(node)
        elif isinstance(node, Lt):
            return self.gen_lt(node)
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
        elif isinstance(node.value, (Eq, Lt)):
            # For comparison operations, generate the code and copy result to var
            result_pos = self.generate(node.value)
            if result_pos != var_pos:
                self.copy_cell(result_pos, var_pos, temp=None)
            
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
        
        # For write syscall, handle string specially
        if node.num == 1 and len(node.args) == 3 and isinstance(node.args[1], String):
            # fd in cell 2
            self.move_ptr(2)
            self.gen_value(node.args[0])
            
            # Store string and put its start position in cell 3
            string_pos = self.gen_string(node.args[1])
            self.move_ptr(3)
            self.set_cell(string_pos)
            
            # Length in cell 4
            self.move_ptr(4)
            self.gen_value(node.args[2])
        else:
            # Set arguments in cells 2-7
            for i, arg in enumerate(node.args):
                if self.debug:
                    self.emit('', f"Setting syscall arg {i}")
                    self.indent_level += 1
                if isinstance(arg, Value):
                    self.move_ptr(2 + i)
                    self.gen_value(arg)
                elif isinstance(arg, Var):
                    # Copy from variable cell to argument cell, preserving the variable
                    var_pos = self.gen_var(arg)
                    self.move_ptr(2 + i)
                    self.copy_cell(var_pos, 2 + i, temp=9)  # Use cell 9 as temp
                elif isinstance(arg, String):
                    # For strings, store the string and put its start position in the arg cell
                    string_pos = self.gen_string(arg)
                    self.move_ptr(2 + i)
                    self.set_cell(string_pos)
                if self.debug:
                    self.indent_level -= 1
        
        # Trigger syscall with cell 0
        # Save current position in cell 9 (temp)
        self.move_ptr(9)
        self.set_cell(self.ptr)  # Store current position
        # Move to cell 0 and trigger syscall
        self.move_ptr(0)  # Move to cell 0
        self.set_cell(255)  # Set cell 0 to 255
        self.emit('.', "Trigger syscall")  # Trigger syscall while at cell 0
        self.emit('[-]', "Clear cell 0")  # Clear cell 0 after syscall
        # Restore position from cell 9
        self.move_ptr(9)
        self.emit('[', "Start restoring position")
        self.emit('>', "Move right")
        self.emit('-', "Decrement counter")
        self.emit(']', "End restoring position")

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

    def gen_block(self, node):
        """Generate code for a block of statements"""
        if self.debug:
            self.emit('', "Block start")
            self.indent_level += 1
            
        for statement in node.statements:
            self.generate(statement)
            
        if self.debug:
            self.indent_level -= 1
            self.emit('', "Block end")

    def gen_if(self, node):
        """
        Generate: if (cond != 0) { then_block }
        else_block is ignored in this minimal example.
        We'll:
          1) generate node.condition => gets a cell condPos containing 0 or 1
          2) use a BF loop to run the 'then_block' if condPos != 0
        """
        if self.debug:
            self.emit('', f"If statement")
            self.indent_level += 1

        # (1) Generate the condition => cell containing 0 or 1
        condPos = self.generate(node.condition)

        # (2) Emit BF loop: if cell condPos != 0, run then-block once, then set condPos=0 to break
        self.move_ptr(condPos)
        self.emit('[', "Start IF loop (cond != 0)")

        # Generate then block
        if isinstance(node.then_block, Node):
            self.generate(node.then_block)
        elif isinstance(node.then_block, list):
            # If user gave a list of statements
            self.generate(Block(node.then_block))

        # Clear cond so we exit the loop
        self.move_ptr(condPos)
        self.emit('[-]', "Clear condition to break IF loop")
        self.emit(']', "End IF loop")

        if self.debug:
            self.indent_level -= 1

        # Return condPos or some cell. Typically, after an if, you don't reuse condPos.
        return condPos

    def gen_eq(self, node):
        """
        eq => 1 if left == right else 0
        We'll store that result in a *newly allocated cell*.
        This version preserves the input values.
        """
        if self.debug:
            self.emit('', f"Eq: {node.left} == {node.right}")
            self.indent_level += 1

        # Evaluate left and right => get their cell positions
        leftPos = self.generate(node.left)
        rightPos = self.generate(node.right)

        # Allocate result and temp cells
        resPos = self.allocate_memory(1)
        tempPos = self.allocate_memory(1)  # For comparison

        # Start with result = 1 (assume equal)
        self.move_ptr(resPos)
        self.set_cell(1)

        # Copy left to temp (preserving left)
        self.copy_cell(leftPos, tempPos, temp=resPos)

        # Now subtract right from temp. If temp becomes 0, they were equal
        self.move_ptr(rightPos)
        self.emit('[', "Start subtracting right")
        self.move_ptr(tempPos)
        self.emit('-', "Decrement temp")
        self.move_ptr(rightPos)
        self.emit('-', "Decrement right")
        self.emit(']', "End subtracting")

        # If temp is not 0, they were not equal
        self.move_ptr(tempPos)
        self.emit('[', "If temp not 0, clear result")
        self.move_ptr(resPos)
        self.set_cell(0)  # Clear result
        self.move_ptr(tempPos)
        self.set_cell(0)  # Clear temp
        self.emit(']', "End temp check")

        # Restore right value
        self.move_ptr(rightPos)
        self.copy_cell(leftPos, rightPos, temp=tempPos)

        if self.debug:
            self.indent_level -= 1

        return resPos

    def gen_lt(self, node):
        """
        lt => 1 if left < right else 0
        This version preserves input values and handles the comparison correctly.
        """
        if self.debug:
            self.emit('', f"Lt: {node.left} < {node.right}")
            self.indent_level += 1

        leftPos = self.generate(node.left)
        rightPos = self.generate(node.right)
        
        # Allocate cells for result and temporaries
        resPos = self.allocate_memory(1)
        tempLeftPos = self.allocate_memory(1)
        tempRightPos = self.allocate_memory(1)

        # Start with result = 0
        self.move_ptr(resPos)
        self.set_cell(0)

        # Copy values to temps to preserve originals
        self.copy_cell(leftPos, tempLeftPos, temp=resPos)
        self.copy_cell(rightPos, tempRightPos, temp=resPos)

        # Decrement both until one hits zero
        self.move_ptr(tempLeftPos)
        self.emit('[', "While tempLeft > 0")
        self.emit('-', "Decrement tempLeft")
        
        # Check if right is zero
        self.move_ptr(tempRightPos)
        self.emit('[', "If tempRight > 0")
        self.emit('-', "Decrement tempRight")
        self.move_ptr(tempLeftPos)
        self.emit(']', "End tempRight check")
        
        self.emit(']', "End tempLeft loop")

        # If tempRight is still nonzero, left < right
        self.move_ptr(tempRightPos)
        self.emit('[', "If tempRight still > 0")
        self.move_ptr(resPos)
        self.set_cell(1)  # Set result to 1
        self.move_ptr(tempRightPos)
        self.set_cell(0)  # Clear tempRight
        self.emit(']', "End final check")

        # Restore original values
        self.move_ptr(rightPos)
        self.copy_cell(leftPos, rightPos, temp=tempLeftPos)

        if self.debug:
            self.indent_level -= 1

        return resPos

class BrainfuckVM:
    def __init__(self, code, memory_size=30000, debug=False, debug_cells=16):
        import sys
        self.sys = sys  # Store sys module as instance variable
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
            
        cells = self.memory[:50]  # Show more cells for debugging
        
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
            print("\n=== Syscall Details ===")
            print(f"Syscall number: {syscall_num}")
            print(f"Arguments: {args[:3]}")  # Show first 3 args
            print("\nMemory state before syscall:")
            self._debug_memory()
            
            # For write syscall, show more details
            if syscall_num == 1:
                fd = args[0]
                buf_start = args[1]
                count = args[2]
                buf = bytes(self.memory[buf_start:buf_start + count])
                print(f"\nWrite syscall details:")
                print(f"  fd: {fd}")
                print(f"  buffer start: {buf_start}")
                print(f"  count: {count}")
                print(f"  buffer content: {[x for x in buf]}")
                try:
                    print(f"  as string: [{buf.decode('utf-8')}]")
                except:
                    print("  as string: [unable to decode]")
                print("\nMemory region for string:")
                print(f"  cells {buf_start}-{buf_start+count-1}: {self.memory[buf_start:buf_start+count]}")
                print("\nFull memory state:")
                self._debug_memory()

        # Filter out zero args from the end
        while args and args[-1] == 0:
            args.pop()

        try:
            # For write syscall, handle it directly
            if syscall_num == 1:  # write
                fd = args[0]
                buf_start = args[1]
                count = args[2]
                
                # Create buffer from our memory
                buf = bytes(self.memory[buf_start:buf_start + count])
                
                # Write directly to stdout/stderr for fd 1/2
                if fd == 1:
                    try:
                        self.sys.stdout.buffer.write(buf)
                        self.sys.stdout.flush()
                        result = count
                    except Exception as e:
                        print(f"Write error: {e}", file=self.sys.stderr)
                        result = 0xFF
                elif fd == 2:
                    try:
                        self.sys.stderr.buffer.write(buf)
                        self.sys.stderr.flush()
                        result = count
                    except Exception as e:
                        print(f"Write error: {e}", file=self.sys.stderr)
                        result = 0xFF
                else:
                    # Get libc
                    if platform.system() == 'Darwin':
                        libc = ctypes.CDLL('libc.dylib')
                    else:
                        libc = ctypes.CDLL('libc.so.6')
                    result = libc.write(fd, buf, count)
                if self.debug:
                    print(f"\nWrite result: {result}")
            else:
                # For other syscalls, use libc
                if platform.system() == 'Darwin':
                    libc = ctypes.CDLL('libc.dylib')
                else:
                    libc = ctypes.CDLL('libc.so.6')
                result = libc.syscall(syscall_num, *args)
                if self.debug:
                    print(f"\nSyscall result: {result}")
                
            self.memory[8] = result & 0xFF  # Store result in cell 8
            if self.debug:
                print("\nMemory state after syscall:")
                self._debug_memory()
        except Exception as e:
            print(f"Syscall error: {e}", file=self.sys.stderr)
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
                if self.debug:
                    print(f"\nExecuting '.': cell[0]={self.memory[0]}, data_ptr={self.data_ptr}")
                
                # Check for syscall - check cell 0 directly
                if self.memory[0] == 255:
                    if self.debug:
                        print("\n=== Syscall Triggered ===")
                        print(f"Cell 0 value at trigger: {self.memory[0]}")
                        print(f"Current data_ptr: {self.data_ptr}")
                        self._debug_memory()
                        print("\n=== Executing Syscall ===")
                    # Save current data pointer
                    saved_ptr = self.data_ptr
                    # Move to cell 0 for syscall
                    self.data_ptr = 0
                    # Handle syscall
                    self._handle_syscall()
                    # Restore data pointer
                    self.data_ptr = saved_ptr
                    if self.debug:
                        print("=== Syscall Complete ===")
                        print(f"Cell 0 value after syscall: {self.memory[0]}")
                        print(f"Current data_ptr: {self.data_ptr}")
                        self._debug_memory()
                else:
                    # Normal output
                    self.sys.stdout.buffer.write(bytes([self.memory[self.data_ptr]]))
                    self.sys.stdout.flush()
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
    program = Block([
        # x = 5
        Assign(Var('x'), Value(5)),
        # y = 10
        Assign(Var('y'), Value(10)),

        # cond = Eq(x, y)
        Assign(Var('cond1'), Eq(Var('x'), Var('y'))),

        # if (cond1) { Syscall write("They are equal!") }
        If(
            Var('cond1'),
            Block([
                # Store string first
                Assign(Var('msg1'), String("They are equal!\n")),
                # Then use it in syscall
                Assign(Var('dummy'), 
                    Syscall(1, [
                        Value(1),            # fd=1
                        Var('msg1'),         # string pointer
                        Value(16)            # length of string
                    ])
                )
            ])
        ),

        # cond = Lt(x, y)
        Assign(Var('cond2'), Lt(Var('x'), Var('y'))),

        # if (cond2) { Syscall write("x < y!") }
        If(
            Var('cond2'),
            Block([
                # Store string first
                Assign(Var('msg2'), String("x < y!\n")),
                # Then use it in syscall
                Assign(Var('dummy2'),
                    Syscall(1, [
                        Value(1),
                        Var('msg2'),         # string pointer
                        Value(7)             # length of string
                    ])
                )
            ])
        ),
    ])

    gen = CodeGenerator(debug=True)
    gen.generate(program)

    print("=== Generated BF code (with debug) ===")
    for code, dbg in gen.output:
        if dbg: 
            print(dbg)
        if code:
            print(code)

    print("\n=== Running in BrainfuckVM ===")
    vm = BrainfuckVM(gen.get_code(), debug=True, debug_cells=50)
    vm.run()
