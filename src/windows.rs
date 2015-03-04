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
	fn run<F: FnOnce(Receiver<State>)>(&self, func: F) -> Result<(), String> {
		let (tx, rx) = channel();
		tx.send(State::Start).unwrap();
		let mut demon = DemonStatic
		{
			tx: Some(tx),
		};
		
		try! (guard_compare_and_swap(demon_null(), &mut demon));
		let result = demon_console(func, rx);
		try! (guard_compare_and_swap(&mut demon, demon_null()));
		result
	}
}

fn guard_compare_and_swap(old_value: *mut DemonStatic, new_value: *mut DemonStatic) -> Result<(), String>
{
	unsafe
	{
		let guard = LOCK.lock().unwrap();
		if demon_static != old_value
		{
			return Err("This function is not reentrant.".to_string());
		}
		demon_static = new_value;
		let _ = guard;
	}
	Ok(())
}

fn demon_console<F: FnOnce(Receiver<State>)>(func: F, rx: Receiver<State>) -> Result<(), String>
{
	unsafe
	{
		if SetConsoleCtrlHandler(Some(console_handler), TRUE) == FALSE
		{
			return Err(format! ("Failed SetConsoleCtrlHandler: {}", error_string(GetLastError() as i32)));
		}
		func(rx);
		if SetConsoleCtrlHandler(Some(console_handler), FALSE) == FALSE
		{
			return Err(format! ("Failed SetConsoleCtrlHandler: {}", error_string(GetLastError() as i32)));
		}
	}
	Ok(())
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
