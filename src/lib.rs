#![feature(collections)]
#![feature(os)]
#![feature(libc)]
#![feature(std_misc)]

pub mod daemon;
pub use daemon::*;

#[cfg(target_os = "windows")]
pub mod windows;
#[cfg(target_os = "windows")]
pub use windows::*;

#[cfg(target_os = "linux")]
pub mod linux;
#[cfg(target_os = "linux")]
pub use linux::*;
