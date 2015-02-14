use super::demon::*;

impl DemonRunner for Demon {
	fn run(&self) {
		println! ("{}", self.name);
	}
}
