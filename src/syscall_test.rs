use std::io;
use syscalls::{syscall, Sysno};

const PORT: u16 = 8080;
const BUFFER_SIZE: usize = 1024;

fn main() -> io::Result<()> {
    println!("Starting echo server on port {}", PORT);

    // Create socket
    let socket_fd = unsafe { syscall!(Sysno::socket, libc::AF_INET, libc::SOCK_STREAM, 0) }? as i32;

    // Set socket options for reuse
    let optval: i32 = 1;
    unsafe {
        syscall!(
            Sysno::setsockopt,
            socket_fd,
            libc::SOL_SOCKET,
            libc::SO_REUSEADDR,
            &optval as *const i32 as usize,
            std::mem::size_of::<i32>()
        )?;
    }

    // Bind socket
    let addr = libc::sockaddr_in {
        sin_family: libc::AF_INET as u16,
        sin_port: PORT.to_be(),
        sin_addr: libc::in_addr { s_addr: 0 },
        sin_zero: [0; 8],
    };

    unsafe {
        syscall!(
            Sysno::bind,
            socket_fd,
            &addr as *const _ as usize,
            std::mem::size_of::<libc::sockaddr_in>()
        )?;
    }

    // Listen
    unsafe {
        syscall!(Sysno::listen, socket_fd, 5)?;
    }

    println!("Server listening on port {}", PORT);

    // Accept loop
    loop {
        let mut client_addr: libc::sockaddr_in = unsafe { std::mem::zeroed() };
        let mut addr_len = std::mem::size_of::<libc::sockaddr_in>() as u32;

        let client_fd = unsafe {
            syscall!(
                Sysno::accept,
                socket_fd,
                &mut client_addr as *mut _ as usize,
                &mut addr_len as *mut _ as usize
            )
        }? as i32;

        println!("Client connected!");

        // Echo loop
        let mut buffer = [0u8; BUFFER_SIZE];
        loop {
            let bytes_read = unsafe {
                syscall!(
                    Sysno::read,
                    client_fd,
                    buffer.as_mut_ptr() as usize,
                    buffer.len()
                )
            }?;

            if bytes_read == 0 {
                break;
            }

            unsafe {
                syscall!(
                    Sysno::write,
                    client_fd,
                    buffer.as_ptr() as usize,
                    bytes_read
                )?;
            }
        }

        // Close client socket
        unsafe {
            syscall!(Sysno::close, client_fd)?;
        }
        println!("Client disconnected");
    }
}
