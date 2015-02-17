#![feature(io)]
#![feature(env)]
#![feature(path)]

extern crate demon;
use demon::Signal;
use demon::Demon;
use demon::DemonRunner;
use std::env;
use std::old_io::{File, Open, ReadWrite, IoError, SeekStyle};
use std::sync::mpsc::Receiver;

fn main() {
	log("Example started.");
	let demon = Demon {
		name: "Example".to_string()
	};
	demon.run(move |rx: Receiver<Signal>| {
		log("Worker started.");
		for signal in rx.iter() {
			match signal {
				Signal::Start => log("Worker: Start"),
				Signal::Reload => log("Worker: Reload"),
				Signal::Shutdown => log("Worker: Shutdown")
			};
		}
		log("Worker finished.");
	});
	log("Example finished.");
}


#[allow(unused_must_use)]
fn log(message: &str) {
	log_safe(message);
}

fn log_safe(message: &str) -> Result<(), IoError> {
	let path = try! (env::current_exe()).with_extension("log");
	let mut file = try! (File::open_mode(&path, Open, ReadWrite));
	try! (file.seek(0, SeekStyle::SeekEnd));
	try! (file.write_line(message));
	println! ("{}", message);
	Ok(())
}
