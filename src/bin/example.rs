#![feature(fs)]
#![feature(io)]
#![feature(env)]
#![feature(path)]

extern crate demon;
use demon::Signal;
use demon::Demon;
use demon::DemonRunner;
use std::env;
use std::fs::OpenOptions;
use std::io::Error;
use std::io::Write;
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

fn log_safe(message: &str) -> Result<(), Error> {
	let path = try! (env::current_exe()).with_extension("log");
	let mut file = try! (OpenOptions::new().append(true).open(&path));
//	try! (file.seek(0, SeekStyle::SeekEnd));
	try! (file.write(message.as_bytes()));
	try! (file.write(b"\n"));
	println! ("{}", message);
	Ok(())
}
