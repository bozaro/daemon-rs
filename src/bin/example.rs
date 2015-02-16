#![feature(io)]
#![feature(env)]
#![feature(path)]

extern crate demon;
use demon::Demon;
use demon::DemonRunner;
use std::env;
use std::old_io::{File, Open, ReadWrite, IoError, SeekStyle};

fn main() {
	log("Example started.");
	let demon = Demon {
		name: "Example".to_string()
	};
	demon.run();
	log("Example finished.");
}


fn log(message: &str) -> Result<(), IoError> {
	let path = try! (env::current_exe()).with_extension("log");
	let mut file = try! (File::open_mode(&path, Open, ReadWrite));
	try! (file.seek(0, SeekStyle::SeekEnd));
	try! (file.write_line(message));
	Ok(())
}
