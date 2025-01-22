// Compare two strings in heap area
// Output:
//  'equals' (61) if equal
//  'gt' (62) if first OR greater
//  'lt' (60) if second OR greater

// Set up syscall parameters in first cells
+              // Syscall 1 (write)
>+             // FD 1 (stdout)
>++++++++     // Buffer at cell(8)
>+             // Length 1

// Set up test strings in heap
>>>>          // Go to cell(7)
++  // First (2)
>++++  // Second (4)

// Compare them
>              // Go to cell(9) for working space
[-]            // Clear the cell
+++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++ // Set to 61 ('equals')

<             // Back to second number
[-<+>]         // Move second to first
<              // Go to first
[              // If first not zero (meaning first was greater)
  >>>>>[-]     // Clear equals sign
  ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++ // Set to 62 ('gt')
  [-<<<<+>>>>] // Move first back
]

<<<<<<.        // Go back to start and execute syscall
