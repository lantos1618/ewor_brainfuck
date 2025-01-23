++>++>+<<.                   # socket(AF_INET, SOCK_STREAM, 0)
>>>> +++                     # Save socket_fd (3) to cell4

>>>>>>>> ++                  # sockaddr_in: sin_family=2 (AF_INET)
>                           # cell8=0 (sin_family padding)
>+++++++++++++++++++++++++   # cell9=31 (sin_port=0x1F)
>+++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++ # cell10=144 (sin_port=0x90)
<<<<<<<<<<<<                 # Return to cell0

>+>>+++++++>>>++++++++<<<<.  # bind(sockfd, addr, sizeof(addr))
++++>+++++<<.                # listen(sockfd, 5)

+++++>.                      # accept client1 (fd4 saved in cell5)
+++++>.                      # accept client2 (fd5 saved in cell6)

# Main loop: read from client1 and send to client2, then vice versa
>+>++++>++++++++<<<<.        # read from client1
+>+++++>++++++++<<<<.        # write to client2
>+>+++++>++++++++<<<<.       # read from client2
+>++++>++++++++<<<<.         # write to client1