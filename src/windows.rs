use super::demon::*;
use std::sync::mpsc::channel;
use std::sync::mpsc::Receiver;

impl DemonRunner for Demon {
	fn run<F: FnOnce(Receiver<Signal>)>(&self, f: F) {
		println! ("{}", self.name);
		let (tx, rx) = channel();
		f(rx);
	}
}
