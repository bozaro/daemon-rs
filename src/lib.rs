#[cfg(target_os = "windows")]
extern crate winapi;

pub mod daemon;
pub use daemon::*;

mod singleton;

#[cfg(target_os = "windows")]
pub mod windows;
#[cfg(target_os = "windows")]
pub use windows::*;

#[cfg(any(target_os = "linux", target_os="macos"))]
pub mod posix;
#[cfg(any(target_os = "linux", target_os="macos"))]
pub use posix::*;
