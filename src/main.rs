use std::num::NonZeroUsize;
use std::sync::{Arc, Mutex};
use std::thread;

use crossbeam::channel;
use failure::Error;
use key_logger::{self, State};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
/// A simple key logger based (very loosely) on xev.
struct Opt {
    /// Port to run http server on (host is always 127.0.0.1)
    #[structopt(short = "p", long = "port")]
    port: Option<u16>,

    /// Turns off printing to stdout.
    #[structopt(short = "q", long = "quiet")]
    quiet: bool,

    /// Number of key presses to keep in memory.
    #[structopt(short = "s", long = "size", default_value = "10")]
    size: usize,
}

fn main() -> Result<(), Error> {
    let opt = Opt::from_args();
    let display = !opt.quiet;

    let size =
        NonZeroUsize::new(opt.size).ok_or_else(|| failure::err_msg("size cannot be zero."))?;

    let state = Arc::new(Mutex::new(State::new(size)));
    let state2 = state.clone();

    let (sender, receiver) = channel::bounded(1);
    let sender2 = sender.clone();

    thread::spawn(move || {
        let result = key_logger::log_keys(display, state);
        sender.send(result).unwrap();
    });

    if let Some(port) = opt.port {
        thread::spawn(move || {
            let result = key_logger::run_server(port, state2);
            sender2.send(result).unwrap();
        });
    }

    receiver.recv()?
}
