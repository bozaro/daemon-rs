extern crate demon;
use demon::Demon;
use demon::DemonRunner;

fn main() {
	let demon = Demon {
		name: "Example".to_string()
	};
	demon.run();
}

