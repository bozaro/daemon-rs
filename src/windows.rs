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
	holder: Box<DemonFunc>,
}

trait DemonFunc
{
	fn exec(&mut self) -> Result<(), String>;
	fn take_tx(&mut self) -> Option<Sender<State>>;
}

struct DemonFuncHolder <F: FnOnce(Receiver<State>)>
{
	tx: Option<Sender<State>>,
	func: Option<(F, Receiver<State>)>,
}


impl <F: FnOnce(Receiver<State>)> DemonFunc for DemonFuncHolder<F>
{
	fn exec(&mut self) -> Result<(), String>
	{
		match self.func.take()
		{
			Some((func, rx)) => {
				func(rx);
				Ok(())
			}
			None => Err(format! ("INTERNAL ERROR: Can't unwrap demon function"))
		}
	}

	fn take_tx(&mut self) -> Option<Sender<State>>
	{
		self.tx.take()
	}
}


impl DemonRunner for Demon
{
	fn run<F: 'static + FnOnce(Receiver<State>)>(&self, func: F) -> Result<(), String> {
		let (tx, rx) = channel();
		tx.send(State::Start).unwrap();
		let mut demon = DemonStatic
		{
			holder: Box::new(DemonFuncHolder
			{
				tx: Some(tx),
				func: Some((func, rx)),
			})
		};
		try! (guard_compare_and_swap(demon_null(), &mut demon));
		let result = demon_console(&mut demon);
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

fn demon_console(demon: &mut DemonStatic) -> Result<(), String>
{
	let result;
	unsafe
	{
		if SetConsoleCtrlHandler(Some(console_handler), TRUE) == FALSE
		{
			return Err(format! ("Failed SetConsoleCtrlHandler: {}", error_string(GetLastError() as i32)));
		}
		result = demon.holder.exec();
		if SetConsoleCtrlHandler(Some(console_handler), FALSE) == FALSE
		{
			return Err(format! ("Failed SetConsoleCtrlHandler: {}", error_string(GetLastError() as i32)));
		}
	}
	result
}

unsafe extern "system" fn console_handler(_: DWORD) -> BOOL
{
	let guard = LOCK.lock().unwrap();
	if demon_static != demon_null()
	{
		return match (*demon_static).holder.take_tx()
		{
			Some(ref tx) => {
				return match tx.send(State::Stop)
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
