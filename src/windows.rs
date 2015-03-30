extern crate winapi;
extern crate advapi32;
extern crate kernel32;

use super::daemon::*;
use std::sync::mpsc::channel;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::sync::StaticMutex;
use std::sync::MUTEX_INIT;
use std::os::error_string;
use std::ptr;
use self::winapi::*;
use self::advapi32::*;
use self::kernel32::*;

static LOCK: StaticMutex = MUTEX_INIT;

static mut daemon_static:*mut DaemonStatic = 0 as *mut DaemonStatic;

struct DaemonStatic
{
	name: String,
	holder: Box<DaemonFunc>,
	handle: SERVICE_STATUS_HANDLE,
}

trait DaemonFunc
{
	fn exec(&mut self) -> Result<(), String>;
	fn send(&mut self, state: State) -> Result<(), String>;
	fn take_tx(&mut self) -> Option<Sender<State>>;
}

struct DaemonFuncHolder <F: FnOnce(Receiver<State>)>
{
	tx: Option<Sender<State>>,
	func: Option<(F, Receiver<State>)>,
}


impl <F: FnOnce(Receiver<State>)> DaemonFunc for DaemonFuncHolder<F>
{
	fn exec(&mut self) -> Result<(), String>
	{
		match self.func.take()
		{
			Some((func, rx)) => {
				func(rx);
				Ok(())
			}
			None => Err(format! ("INTERNAL ERROR: Can't unwrap daemon function"))
		}
	}

	fn send(&mut self, state: State) -> Result<(), String>
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
	fn run<F: 'static + FnOnce(Receiver<State>)>(&self, func: F) -> Result<(), String> {
		let (tx, rx) = channel();
		tx.send(State::Start).unwrap();
		let mut daemon = DaemonStatic
		{
			name: self.name.clone(),
			holder: Box::new(DaemonFuncHolder
			{
				tx: Some(tx),
				func: Some((func, rx)),
			}),
			handle: 0 as SERVICE_STATUS_HANDLE,
		};
		try! (guard_compare_and_swap(daemon_null(), &mut daemon));
		let result = daemon_service(&mut daemon);
		try! (guard_compare_and_swap(&mut daemon, daemon_null()));
		result
	}
}

fn guard_compare_and_swap(old_value: *mut DaemonStatic, new_value: *mut DaemonStatic) -> Result<(), String>
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

fn daemon_service(daemon: &mut DaemonStatic) -> Result<(), String>
{
	unsafe
	{
		let service_name = service_name(&daemon.name);
		let service_table: &[*const SERVICE_TABLE_ENTRYW] = &[
			&SERVICE_TABLE_ENTRYW {
				lpServiceName: service_name.as_ptr(),
				lpServiceProc: Some(service_main),
			},
			ptr::null()
		];
		match StartServiceCtrlDispatcherW(*service_table.as_ptr())
		{
			0 => daemon_console(daemon),
			_ => Ok(())
		}
	}
}

fn daemon_console(daemon: &mut DaemonStatic) -> Result<(), String>
{
	let result;
	unsafe
	{
		if SetConsoleCtrlHandler(Some(console_handler), TRUE) == FALSE
		{
			return Err(format! ("Failed SetConsoleCtrlHandler: {}", error_string(GetLastError() as i32)));
		}
		result = daemon.holder.exec();
		if SetConsoleCtrlHandler(Some(console_handler), FALSE) == FALSE
		{
			return Err(format! ("Failed SetConsoleCtrlHandler: {}", error_string(GetLastError() as i32)));
		}
	}
	result
}

fn service_name(name: &str) -> Vec<u16> {
	let mut result: Vec<u16> = name.utf16_units().collect();
	result.push(0);
	result
}

fn create_service_status(current_state: DWORD) -> SERVICE_STATUS {
	SERVICE_STATUS {
		dwServiceType: SERVICE_WIN32_OWN_PROCESS,
		dwCurrentState: current_state,
		dwControlsAccepted: SERVICE_ACCEPT_STOP | SERVICE_ACCEPT_SHUTDOWN,
		dwWin32ExitCode: 0,
		dwServiceSpecificExitCode: 0,
		dwCheckPoint: 0,
		dwWaitHint: 0,
	}
}

unsafe extern "system" fn service_main(
	_: DWORD, // dw_num_services_args
	_: *mut LPWSTR, // lp_service_arg_vectors
) {
	let guard = LOCK.lock().unwrap();
	if daemon_static != daemon_null()
	{
		let daemon = &mut *daemon_static;
		let service_name = service_name(&daemon.name);
		daemon.handle = RegisterServiceCtrlHandlerExW(service_name.as_ptr(), Some(service_handler), ptr::null_mut());
		SetServiceStatus (daemon.handle, &mut create_service_status(SERVICE_START_PENDING));
		SetServiceStatus (daemon.handle, &mut create_service_status(SERVICE_RUNNING));

		daemon.holder.exec().unwrap();
		SetServiceStatus (daemon.handle, &mut create_service_status(SERVICE_STOPPED));
	}
	let _ = guard;
}

unsafe extern "system" fn service_handler(
	dw_control: DWORD,
	_: DWORD,  // dw_event_type
	_: LPVOID, // lp_event_data
	_: LPVOID  // lp_context
) -> DWORD {
	let daemon = &mut *daemon_static;
	match dw_control {
		SERVICE_CONTROL_STOP | SERVICE_CONTROL_SHUTDOWN => {
			match daemon.holder.take_tx()
			{
				Some(ref tx) => {
					SetServiceStatus (daemon.handle, &mut create_service_status(SERVICE_STOP_PENDING));
					let _ = tx.send(State::Stop);
				}
				None => {}
			}
		}
		_ => {}
	};
	0
}

unsafe extern "system" fn console_handler(_: DWORD) -> BOOL
{
	let guard = LOCK.lock().unwrap();
	if daemon_static != daemon_null()
	{
		let daemon = &mut *daemon_static;
		return match daemon.holder.take_tx()
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

fn daemon_null() -> *mut DaemonStatic {
	0 as *mut DaemonStatic
}
