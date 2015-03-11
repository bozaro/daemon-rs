#![feature(collections)]
#![feature(os)]
#![feature(std_misc)]

pub mod demon;
pub use demon::*;

#[cfg(target_os = "windows")]
pub mod windows;
#[cfg(target_os = "windows")]
pub use windows::*;

#[cfg(target_os = "linux")]
pub mod linux;
#[cfg(target_os = "linux")]
pub use linux::*;
