use std::ops::{Deref, DerefMut};
use std::os::unix::net::UnixStream;
use bincode::config::{BigEndian, Configuration, Fixint};
use tokio::sync::mpsc::Sender;
use shared::{ConnectionType, External};
use crate::io::Internal;

#[derive(Default)]
pub struct Clients(Vec<Client>);

impl Clients {
    pub fn send(&mut self, msg: &External) {
        for client in self.0.iter_mut() {
            bincode::encode_into_std_write(msg, &mut client.stream, client.config).unwrap();
        }
    }
}

impl Deref for Clients {
    type Target = Vec<Client>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Clients {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub struct Client {
    client_type: ConnectionType,
    stream: UnixStream,
    internal_tx: Sender<Internal>,
    config: Configuration<BigEndian, Fixint>,
}

impl Client {
    pub fn new(client_type: ConnectionType, stream: UnixStream, internal_tx: Sender<Internal>) -> Self {
        Self {
            client_type,
            stream,
            internal_tx,
            config: bincode::config::standard().with_big_endian().with_fixed_int_encoding(),
        }
    }

    pub fn test(&mut self) {

    }
}