#![feature(io)]
#![feature(path)]

extern crate daemon;
extern crate time;

use daemon::State;
use daemon::Daemon;
use daemon::DaemonRunner;
use std::env;
use std::fs::OpenOptions;
use std::io::Error;
use std::io::Write;
use std::sync::mpsc::Receiver;

fn main() {
	log("Example started.");
	let daemon = Daemon {
		name: "example".to_string()
	};
	daemon.run(move |rx: Receiver<State>| {
		log("Worker started.");
		for signal in rx.iter() {
			match signal {
				State::Start => log("Worker: Start"),
				State::Reload => log("Worker: Reload"),
				State::Stop => log("Worker: Stop")
			};
		}
		log("Worker finished.");
	}).unwrap();
	log("Example finished.");
}


#[allow(unused_must_use)]
fn log(message: &str) {
	log_safe(message);
}

fn log_safe(message: &str) -> Result<(), Error> {
	let line = format! ("{}: {}", time::strftime("%Y-%m-%d %H:%M:%S", &time::now()).unwrap(), message);
	println! ("{}", line);
	let path = try! (env::current_exe()).with_extension("log");
	let mut file = try! (OpenOptions::new().create(true).write(true).append(true).open(&path));
	try! (file.write(line.as_bytes()));
	try! (file.write(b"\n"));
	Ok(())
}
