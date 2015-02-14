pub enum Signal {
	// Need to reload settings
	Reload
}

pub struct Demon {
	// Demon name
	pub name: String,
}

pub trait DemonRunner {
	fn run(&self);
}
