extern crate winapi;
extern crate "kernel32-sys" as kernel32;

use super::demon::*;
use std::sync::mpsc::channel;
use std::sync::mpsc::Receiver;
use std::os::error_string;
use self::winapi::*;
use self::kernel32::*;

impl DemonRunner for Demon {
	fn run<F: FnOnce(Receiver<Signal>)>(&self, f: F) {
		unsafe
		{
			if SetConsoleCtrlHandler(Some(console_handler), TRUE) == 0
			{
				panic! ("SetConsoleCtrlHandler: {}", error_string(GetLastError() as i32));
			}
			println! ("{}", self.name);
			let (tx, rx) = channel();
			f(rx);
		}
	}
}

unsafe extern "system" fn console_handler(ctrl_type: DWORD) -> BOOL
{
	match ctrl_type
	{
		CTRL_C_EVENT => println! ("Handler: CTRL_C_EVENT"),
		CTRL_BREAK_EVENT => println! ("Handler: CTRL_BREAK_EVENT"),
		CTRL_CLOSE_EVENT => println! ("Handler: CTRL_CLOSE_EVENT"),
		CTRL_LOGOFF_EVENT => println! ("Handler: CTRL_LOGOFF_EVENT"),
		CTRL_SHUTDOWN_EVENT => println! ("Handler: CTRL_SHUTDOWN_EVENT"),
		_ => println! ("Handler: {}", ctrl_type)
	}	
	FALSE
}
