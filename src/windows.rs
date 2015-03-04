extern crate winapi;
extern crate "kernel32-sys" as kernel32;

use super::demon::*;
use std::sync::mpsc::channel;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::sync::StaticMutex;
use std::sync::MUTEX_INIT;
use std::os::error_string;
use self::winapi::*;
use self::kernel32::*;

static LOCK: StaticMutex = MUTEX_INIT;

static mut demon_static:*mut DemonStatic = 0 as *mut DemonStatic;

struct DemonStatic
{
	tx: Option<Sender<State>>,
}

impl DemonRunner for Demon
{
	fn run<F: FnOnce(Receiver<State>)>(&self, f: F) {
		let (tx, rx) = channel();
		tx.send(State::Start).unwrap();
		let mut demon = DemonStatic
		{
			tx: Some(tx),
		};

		unsafe
		{
			let guard = LOCK.lock().unwrap();
			if demon_static != demon_null()
			{
				panic! ("This function is not reentrant.");
			}
			demon_static = &mut demon;
			let _ = guard;
		}
		
		unsafe
		{
			if SetConsoleCtrlHandler(Some(console_handler), TRUE) == FALSE
			{
				panic! ("SetConsoleCtrlHandler: {}", error_string(GetLastError() as i32));
			}
			f(rx);
			if SetConsoleCtrlHandler(Some(console_handler), FALSE) == FALSE
			{
				panic! ("SetConsoleCtrlHandler: {}", error_string(GetLastError() as i32));
			}
		}

		unsafe
		{
			let guard = LOCK.lock().unwrap();
			demon_static = demon_null();
			let _ = guard;
		}
	}
}

unsafe extern "system" fn console_handler(_: DWORD) -> BOOL
{
	let guard = LOCK.lock().unwrap();
	if demon_static != demon_null()
	{
		return match (*demon_static).tx
		{
			Some(ref tx) => {
				let result = tx.send(State::Stop);
				(*demon_static).tx = None;
				return match result
				{
					Ok(_) => TRUE,
					Err(_) => FALSE,
				}
			}
			None => TRUE
		}
	}
	let _ = guard;
	FALSE
}

fn demon_null() -> *mut DemonStatic {
	0 as *mut DemonStatic
}
