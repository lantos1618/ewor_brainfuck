- Build a server and client for a 1:1 chat application in Brainfuck


I'm guessing they want me to use sys calls. I can not do this 1:1 chat with only brainfuck; lol.


1:1 meaning one to one.
meaning client1 <-> client2

right?

so I can make it

```c

host:    
    sock = socket(AF_INET, SOCK_STREAM, 0);
    bind(sock, (struct sockaddr *)&addr, sizeof(addr));
    listen(sock, 10);

    // 1:1 chat we want to be able to non-blocking litle harder
    client_sock = accept(sock, NULL, NULL);
    while (1) {
        send(client_sock, "Hello, client!", 14, 0);
        recv(client_sock, buffer, sizeof(buffer), 0);
        printf("%s", buffer);
    }

    close(client_sock);
    close(sock);

join:
    sock = socket(AF_INET, SOCK_STREAM, 0);
    client_sock = connect(sock, (struct sockaddr *)&addr, sizeof(addr));

    while (1) {
        recv(client_sock, buffer, sizeof(buffer), 0);
        printf("%s", buffer);
        send(client_sock, "Hello, host!", 14, 0);
    }
```




need to modify BF to do sys calls


[0, 0, 0, 0...]

hijack first cell,

match cells[0]
 | 1 => we do sys call
 | 0 => we do not do sys call, normal BF



match char
    '>' => move right
    '<' => move left
    '+' => increment
    '-' => decrement
    '.' => output | run sys call
    ',' => input
    '[' => loop start
    ']' => loop end


sys_call cells table

0 => is_sys_call
1 => sys_call_id
2 => sys_call_arg1
3 => sys_call_arg2
4 => sys_call_arg3
5 => sys_call_arg4
6 => sys_call_arg5
7 => sys_call_arg6
8 => sys_call_arg7
9 => sys_call_arg8
10 => sys_call_arg9
10 => sys_call_arg10


cells[0] = rax (syscall number)
cells[1] = rdi (arg1)
cells[2] = rsi (arg2) 
cells[3] = rdx (arg3)
cells[4] = r10 (arg4)
cells[5] = r8  (arg5)
cells[6] = r9  (arg6)


yo lets call this BFA

brainfuckassembler

