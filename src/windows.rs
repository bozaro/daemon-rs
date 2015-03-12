extern crate winapi;
extern crate "advapi32-sys" as advapi32;
extern crate "kernel32-sys" as kernel32;

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

static mut demon_static:*mut DemonStatic = 0 as *mut DemonStatic;

struct DemonStatic
{
	name: String,
	holder: Box<DemonFunc>,
	handle: SERVICE_STATUS_HANDLE,
}

trait DemonFunc
{
	fn exec(&mut self) -> Result<(), String>;
	fn send(&mut self, state: State) -> Result<(), String>;
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


impl DemonRunner for Demon
{
	fn run<F: 'static + FnOnce(Receiver<State>)>(&self, func: F) -> Result<(), String> {
		let (tx, rx) = channel();
		tx.send(State::Start).unwrap();
		let mut demon = DemonStatic
		{
			name: self.name.clone(),
			holder: Box::new(DemonFuncHolder
			{
				tx: Some(tx),
				func: Some((func, rx)),
			}),
			handle: 0 as SERVICE_STATUS_HANDLE,
		};
		try! (guard_compare_and_swap(demon_null(), &mut demon));
		let result = demon_service(&mut demon);
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

fn demon_service(demon: &mut DemonStatic) -> Result<(), String>
{
	unsafe
	{
		let service_name = service_name(&demon.name);
		let service_table: &[*const SERVICE_TABLE_ENTRYW] = &[
			&SERVICE_TABLE_ENTRYW {
				lpServiceName: service_name.as_ptr(),
				lpServiceProc: Some(service_main),
			},
			ptr::null()
		];
		match StartServiceCtrlDispatcherW(*service_table.as_ptr())
		{
			0 => demon_console(demon),
			_ => Ok(())
		}
	}
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
	if demon_static != demon_null()
	{
		let demon = &mut *demon_static;
		let service_name = service_name(&demon.name);
		demon.handle = RegisterServiceCtrlHandlerExW(service_name.as_ptr(), Some(service_handler), ptr::null_mut());
		SetServiceStatus (demon.handle, &mut create_service_status(SERVICE_START_PENDING));
		SetServiceStatus (demon.handle, &mut create_service_status(SERVICE_RUNNING));

		demon.holder.exec().unwrap();
		SetServiceStatus (demon.handle, &mut create_service_status(SERVICE_STOPPED));
	}
	let _ = guard;
}

unsafe extern "system" fn service_handler(
	dw_control: DWORD,
	_: DWORD,  // dw_event_type
	_: LPVOID, // lp_event_data
	_: LPVOID  // lp_context
) -> DWORD {
	let demon = &mut *demon_static;
	match dw_control {
		SERVICE_CONTROL_STOP | SERVICE_CONTROL_SHUTDOWN => {
			match demon.holder.take_tx()
			{
				Some(ref tx) => {
					SetServiceStatus (demon.handle, &mut create_service_status(SERVICE_STOP_PENDING));
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
	if demon_static != demon_null()
	{
		let demon = &mut *demon_static;
		return match demon.holder.take_tx()
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
