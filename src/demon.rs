use std::sync::mpsc::Receiver;

pub enum State {
	Start,
	Reload,
	Stop,
}

pub struct Demon {
	// Demon name
	pub name: String,
}

pub trait DemonRunner {
	fn run<F: FnOnce(Receiver<State>)>(&self, f: F) -> Result<(), String>;
}
