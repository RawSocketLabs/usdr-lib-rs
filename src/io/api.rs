use std::os::unix::net::{UnixListener, UnixStream};
use std::thread;
use sdr::FreqBlock;
use tokio::sync::{mpsc::{Receiver, Sender}, watch::Receiver as WatchReceiver};
use crate::io::{External, Input, Output};

pub fn start(in_tx: Sender<Input>, out_rx: Receiver<Output>, realtime_rx: WatchReceiver<FreqBlock>) {
    thread::spawn(move || {
        let mut connected = 0;
        let mut listener = UnixListener::bind("/tmp/sdrscanner").unwrap();


        match listener.accept() {
            Ok((stream, addr)) => handle_client(stream, in_tx, out_rx, realtime_rx),
            _ => {}
        }
    });
}

fn handle_client(mut stream: UnixStream, in_tx: Sender<Input>, out_rx: Receiver<Output>, mut realtime_rx: WatchReceiver<FreqBlock>) {
    let mut config = bincode::config::standard().with_big_endian().with_fixed_int_encoding();

    if let External::Connection(ctype) = bincode::decode_from_std_read(&mut stream, config).unwrap() {
        println!("GOT CONNECTION OF TYPE: {:?}", ctype);
    }

    loop {
        let freq_block = realtime_rx.borrow_and_update().clone();
        bincode::encode_into_std_write(freq_block, &mut stream, config).unwrap();
        while !realtime_rx.has_changed().unwrap() {}
    }


}
