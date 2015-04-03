extern crate libc;

use super::daemon::*;
use std::io::{Error, ErrorKind};
use std::sync::mpsc::channel;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::sync::StaticMutex;
use std::sync::MUTEX_INIT;

static LOCK: StaticMutex = MUTEX_INIT;

static mut daemon_static:*mut DaemonStatic = 0 as *mut DaemonStatic;

struct DaemonStatic
{
	holder: Box<DaemonFunc>,
}

trait DaemonFunc
{
	fn exec(&mut self) -> Result<(), Error>;
	fn send(&mut self, state: State) -> Result<(), Error>;
	fn take_tx(&mut self) -> Option<Sender<State>>;
}

struct DaemonFuncHolder <F: FnOnce(Receiver<State>)>
{
	tx: Option<Sender<State>>,
	func: Option<(F, Receiver<State>)>,
}


impl <F: FnOnce(Receiver<State>)> DaemonFunc for DaemonFuncHolder<F>
{
	fn exec(&mut self) -> Result<(), Error>
	{
		match self.func.take()
		{
			Some((func, rx)) => {
				func(rx);
				Ok(())
			}
			None => Err(Error::new(ErrorKind::Other, "INTERNAL ERROR: Can't unwrap daemon function"))
		}
	}

	fn send(&mut self, state: State) -> Result<(), Error>
	{
		match self.tx
		{
			Some(ref tx) => match tx.send(state)
			{
				Err(e) => Err(format! ("Send new state error: {:?}", e)),
				Ok(_) => Ok(()),
			},
			None => Err(format! ("Service is already exited")),
		}
	}
	
	fn take_tx(&mut self) -> Option<Sender<State>>
	{
		self.tx.take()
	}
}


impl DaemonRunner for Daemon
{
	fn run<F: 'static + FnOnce(Receiver<State>)>(&self, func: F) -> Result<(), Error> {
		let (tx, rx) = channel();
		tx.send(State::Start).unwrap();
		let mut daemon = DaemonStatic
		{
			holder: Box::new(DaemonFuncHolder
			{
				tx: Some(tx),
				func: Some((func, rx)),
			}),
		};
		try! (guard_compare_and_swap(daemon_null(), &mut daemon));
		let result = daemon_console(&mut daemon);
		try! (guard_compare_and_swap(&mut daemon, daemon_null()));
		result
	}
}

fn guard_compare_and_swap(old_value: *mut DaemonStatic, new_value: *mut DaemonStatic) -> Result<(), Error>
{
	unsafe
	{
		let guard = LOCK.lock().unwrap();
		if daemon_static != old_value
		{
			return Err("This function is not reentrant.".to_string());
		}
		daemon_static = new_value;
		let _ = guard;
	}
	Ok(())
}

fn daemon_console(daemon: &mut DaemonStatic) -> Result<(), Error>
{
	let result;
	unsafe
	{
		//if SetConsoleCtrlHandler(Some(console_handler), TRUE) == FALSE
		//{
		//	return Err(format! ("Failed SetConsoleCtrlHandler: {}", error_string(GetLastError() as i32)));
		//}
		result = daemon.holder.exec();
		//if SetConsoleCtrlHandler(Some(console_handler), FALSE) == FALSE
		//{
		//	return Err(format! ("Failed SetConsoleCtrlHandler: {}", error_string(GetLastError() as i32)));
		//}
	}
	result
}

unsafe extern "system" fn signal_handler(sig: libc::c_int)
{
	let guard = LOCK.lock().unwrap();
	if daemon_static != daemon_null()
	{
		let daemon = &mut *daemon_static;
		return match daemon.holder.take_tx()
		{
			Some(ref tx) => {
				let _ = tx.send(State::Stop);
			}
			None => ()
		}
	}
	let _ = guard;
}

fn daemon_null() -> *mut DaemonStatic {
	0 as *mut DaemonStatic
}

extern "C" {
	fn raise(sig: libc::c_int) -> libc::c_int;
	fn signal(sig: libc::c_int, handler: *const libc::c_void) -> libc::c_int;
	fn kill(pid: libc::pid_t, sig: libc::c_int) -> libc::c_int;
}
