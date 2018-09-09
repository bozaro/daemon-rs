use std::io::Error;
use std::sync::mpsc::Receiver;

pub enum State {
    Start,
    Reload,
    Stop,
}

pub struct Daemon {
    // Daemon name
    pub name: String,
}

pub trait DaemonRunner {
    fn run<F: 'static + FnOnce(Receiver<State>)>(&self, f: F) -> Result<(), Error>;
}
