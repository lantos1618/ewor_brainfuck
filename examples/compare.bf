// Compare two numbers and store result in heap
// Result meanings:
//  'equals' (61) = equal
//  'gt' (62) = first number greater
//  'lt' (60) = second number greater

// Set up test numbers in heap (starting at cell(7))
>>>>>>>
+++++          // cell(7) = 5  (first number)
>+++           // cell(8) = 3  (second number)
>              // cell(9) will be our working copy of first number
>              // cell(10) will be working copy of second number
>              // cell(11) will store our result character

// Copy numbers to working area
<<< [>> + << -]  // Copy cell(7) to cell(9)
> [>> + << -]    // Copy cell(8) to cell(10)

// Restore original numbers
<<< [>> + << -]  // Restore cell(7)
> [>> + << -]    // Restore cell(8)

// Go to working cells and compare
>> // Now at cell(9)

// Compare loop - decrement both until one hits zero
[
  -            // Decrement first number
  >            // Move to second number
  [            // If second number is not zero
    -          // Decrement it
    >+         // Increment result cell (cell(11))
    <          // Back to second number
  ]
  <            // Back to first number
]

// If first number is zero check if second had any left
>              // Go to second number
[
  -            // Decrement remaining
  >++          // Add 2 to result cell
  <            // Back to second number
]

// Convert result to character:
// 0 becomes 'equals' (61)
// 1 becomes 'gt' (62)
// 2 becomes 'lt' (60)
>              // Go to result cell (11)
+++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++ // Set to 61 (equals)
// If result was 1 increment once more to 'gt'
// If result was 2 decrement once to 'lt'

// Go back to start for syscall setup
<<<<<<<<<<<    // Back to cell(0) from cell(11)

// Set up write syscall
+               // syscall 1 (write) in cell(0)
>+              // fd 1 (stdout) in cell(1)
>++++++++++     // buffer pointer (11) in cell(2)
>+              // length 1 in cell(3)

// Execute write syscall
<<<.            // Back to cell(0) and execute 