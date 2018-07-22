extern crate daemon;

use daemon::Daemon;
use daemon::DaemonRunner;
use daemon::State;
use std::env;
use std::fs::OpenOptions;
use std::io::Error;
use std::io::Write;
use std::sync::mpsc::Receiver;

fn main() -> Result<(), Error> {
    log("Example started.");
    let daemon = Daemon {
        name: "example".to_string(),
    };
    daemon.run(move |rx: Receiver<State>| {
        log("Worker started.");
        for signal in rx.iter() {
            match signal {
                State::Start => log("Worker: Start"),
                State::Reload => log("Worker: Reload"),
                State::Stop => log("Worker: Stop"),
            };
        }
        log("Worker finished.");
    })?;
    log("Example finished.");

    Ok(())
}

fn log(message: &str) {
    let _ = log_safe(message);
}

fn log_safe(message: &str) -> Result<(), Error> {
    println!("{}", message);
    let path = env::current_exe()?.with_extension("log");
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(&path)?;
    file.write(message.as_bytes())?;
    file.write(b"\n")?;
    Ok(())
}
