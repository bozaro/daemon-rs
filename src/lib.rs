#![feature(collections)]
#![feature(os)]
#![feature(std_misc)]

pub mod demon;
pub use demon::*;

#[cfg(windows)]
pub mod windows;
#[cfg(windows)]
pub use windows::*;
