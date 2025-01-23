Memory Layout Plan
cell0 = syscall number for socket accept read write
cell1 = socket fd and client fd
cell2 = buffer pointer for read write must be HEAP_START or greater
cell3 = length for read write
cell4 = socket options
cell5 = port number we will use 8080
cell6 to 7 = working space for comparisons
cell8 onwards = buffer space for received data

Program Flow Steps
Step 1 Create socket with syscall 2
Step 2 Bind to port with syscall 3
Step 3 Listen for connections with syscall 4
Step 4 Accept connection with syscall 5
Step 5 Read write loop
       Read from client syscall 0
       Write back to client syscall 1
       Compare for exit condition
Step 6 Close connection syscall 7
Step 7 Return to accept new connections

End of comments

// Clear all cells first
>[-]            Clear cell1
>[-]            Clear cell2
>[-]            Clear cell3
>[-]            Clear cell4
>[-]            Clear cell5
>[-]            Clear cell6
>[-]            Clear cell7
<<<<<<<         Back to cell0
[-]             Clear cell0

// Create socket
++              Set syscall to 2 socket
>++             Set domain AF_INET 2
>+              Set type SOCK_STREAM 1
>               Protocol is already 0
<<<             Back to cell0

// Debug - Print current state
.               Print syscall number (cell0)
>.              Print domain (cell1)
>.              Print type (cell2)
>.              Print protocol (cell3)
<<<             Back to cell0

// Execute socket syscall
.               Execute socket syscall

// Debug - Print result
>.              Print result
<               Back to cell0

// Now let's print the final state
>>.             Print syscall number (cell0)
>.              Print domain (cell1)
>.              Print type (cell2)
>.              Print protocol (cell3)
<<<<            Back to start

// Save socket fd to cell4 and preserve it
>[-]            Clear cell1
<[>+<]          Move socket fd to cell1

// Debug - Print socket fd
>.              Print socket fd (cell1)
<               Back to start

>>>>[-]         Clear cell4
<<<<[>>>>+<<<<] Move socket fd to cell4
>[-]            Clear cell1
<[>+<]          Move socket fd back to cell0

// Add setsockopt syscall
[-]             Clear syscall number
++++++++++++++  Set syscall to 14 setsockopt
>[-]            Clear cell1
<[>+<]          Move socket fd to cell1
>+              Set level to SOL_SOCKET (1)
>++             Set optname to SO_REUSEADDR (2)
>++++++++       Set optval pointer to HEAP_START (8)
>++++           Set optlen to 4
[<]             Reset to start

// Debug - Print setsockopt setup
>>.             Print syscall number (cell0)
>.              Print socket fd (cell1)
>.              Print level (cell2)
>.              Print optname (cell3)
<<<<            Back to start

.               Execute setsockopt syscall

// Save socket fd back to cell4
>[-]            Clear cell1
<[>+<]          Move setsockopt result to cell1
>>>>[-]         Clear cell4
<<<<<[>>>>+<<<<] Move socket fd back to cell4

// Setup optval at HEAP_START
>>>>>>>>[-]     Clear cell at HEAP_START
+               Set optval to 1
<<<<<<<<        Back to start

// Setup bind syscall
[-]             Clear syscall number
+++             Set syscall to 3 bind

// Setup sockaddr_in structure at HEAP_START first
>>>>>>>>        Move to HEAP_START cell 8
[-]             Clear family
++              Set family AF_INET 2
>[-]            Clear padding
>[-]            Clear port high byte
++++++++        Port high byte 8 (for port 2048)
>[-]            Clear port low byte
                Port low byte 0 (for port 2048)
>[-]            Clear first addr byte
>[-]            Clear second addr byte
>[-]            Clear third addr byte
>[-]            Clear fourth addr byte
[<]             Reset to start

// Now setup bind syscall args in cells 1 2 3
>[-]            Clear cell1
>>>>>[<<<<+>>>>>]  Move socket fd from cell4 to cell1
>[-]            Clear cell2
+++++++         Set buffer pointer to HEAP_START 7
>[-]            Clear cell3
++++++++++++++++  Set length to 16
[<]             Reset to start

// Debug - Print bind setup
>>.             Print syscall number (cell0)
>.              Print socket fd (cell1)
>.              Print buffer pointer (cell2)
>.              Print length (cell3)
<<<<            Back to start

.               Execute bind syscall
>[-]            Clear cell1
<[>+<]          Move bind result to cell1
>>>>[-]         Clear cell4
<<<<<[>>>>+<<<<] Move socket fd back to cell4

[               If bind succeeded
  Setup listen
  [-]           Clear syscall number
  ++++          Set syscall to 4 listen
  >[<]          Move socket fd to cell1
  >+            Set backlog to 1
  [<]           Reset to start
  .             Execute listen syscall
  >[-]          Clear cell1 before moving result
  <[->+<]       Move listen result to cell1 and preserve it
  >             Move to cell1

  [             If listen succeeded
    Accept loop start
    [<]         Reset to start
    +++++       Set syscall to 5 accept
    >[<]        Move socket fd to cell1
    >++++++++   Set client addr buffer to HEAP_START
    >++         Set length to 2
    [<]         Reset to start
    .           Execute accept syscall
    >[-]        Clear cell1 before moving result
    <[->+<]     Move accept result (client fd) to cell1
    >           Move to cell1

    [           If accept succeeded
      Read write loop
      [           Start loop for handling client
        [<]       Reset to start
        [-]       Clear syscall number for read
        >[<]      Move client fd to cell1
        >++++++++  Set buffer to HEAP_START for read
        >+         Set length to 1
        [<]       Reset to start
        .         Execute read syscall 0
        >[-]      Clear cell1 before moving result
        <[->+<]   Move read result to cell1
        >         Move to cell1

        [         If read succeeded nonzero result
          [<]     Reset to start
          +       Set syscall to 1 write
          >[<]    Move client fd to cell1
          >++++++++  Set buffer to HEAP_START
          >+        Set length to 1
          [<]     Reset to start
          .       Execute write syscall
          >[-]    Clear cell1 before moving result
          <[->+<] Move write result to cell1
        ]
        [<]     Reset to start
      ]

      Close client
      [<]       Reset to start
      +++++++   Set syscall to 7 close
      >[<]      Move client fd to cell1
      [<]       Reset to start
      .         Execute close syscall
      >[-]      Clear cell1 before moving result
      <[->+<]   Move close result to cell1
    ]           End accept success check
    [<]         Reset to start
  ]             End listen success check
]               End bind success check

Return to accept loop
[<]             Reset to start 