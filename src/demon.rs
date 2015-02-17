use std::sync::mpsc::Receiver;

pub enum Signal {
	// Last state
	Start,
	// Need to reload settings
	Reload,
	// Last state
	Shutdown,
}

pub struct Demon {
	// Demon name
	pub name: String,
}

pub trait DemonRunner {
	fn run<F: FnOnce(Receiver<Signal>)>(&self, f: F);
}
