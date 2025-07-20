//! Portable syscall constants that work across Linux and macOS

#[cfg(target_os = "linux")]
pub const SYS_WRITE: i32 = 1;
#[cfg(target_os = "macos")]
pub const SYS_WRITE: i32 = 4;

#[cfg(target_os = "linux")]
pub const SYS_READ: i32 = 0;
#[cfg(target_os = "macos")]
pub const SYS_READ: i32 = 3;

#[cfg(target_os = "linux")]
pub const SYS_CLOSE: i32 = 3;
#[cfg(target_os = "macos")]
pub const SYS_CLOSE: i32 = 6;

#[cfg(target_os = "linux")]
pub const SYS_SOCKET: i32 = 41;
#[cfg(target_os = "macos")]
pub const SYS_SOCKET: i32 = 97;

#[cfg(target_os = "linux")]
pub const SYS_BIND: i32 = 49;
#[cfg(target_os = "macos")]
pub const SYS_BIND: i32 = 104;

#[cfg(target_os = "linux")]
pub const SYS_LISTEN: i32 = 50;
#[cfg(target_os = "macos")]
pub const SYS_LISTEN: i32 = 106;

#[cfg(target_os = "linux")]
pub const SYS_ACCEPT: i32 = 43;
#[cfg(target_os = "macos")]
pub const SYS_ACCEPT: i32 = 30;

// Common constants
pub const AF_INET: i32 = 2;
pub const SOCK_STREAM: i32 = 1; 