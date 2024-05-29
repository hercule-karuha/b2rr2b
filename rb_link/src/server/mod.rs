use crate::config::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::io::{Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::sync::{atomic::AtomicU32, Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::{fs, usize};

mod getter;
pub use getter::*;

#[derive(Serialize, Deserialize)]
pub enum GetPutMessage {
    Get(u32),
    Put(B2RMessage),
}

/// Message send to Bluesim:
/// - id: ID of the probe that the message will send to
/// - message: the message to send, Please ensure that message.len() == ceil(get_t_width/8), where get_t_width is the width of get_t defined in your BSV code.
#[derive(Serialize, Deserialize)]
pub struct R2BMessage {
    pub id: u32,
    pub message: Vec<u8>,
}

/// Message from Bluesim:
/// - id: ID of the probe that sent the message
/// - cycles: Clock cycles when the message was sent
/// - message: Sent message, where message.len() == ceil(put_t_width/8). put_t_width is the width of put_t defined in your BSV code.
#[derive(Serialize, Deserialize, Clone)]
pub struct B2RMessage {
    pub id: u32,
    pub cycles: u32,
    pub message: Vec<u8>,
}

/// A server for interacting with Bluesim.
/// Cache bidirectional data and send data upon receiving requests.
pub struct B2RServer {
    socket_path: String,
    cycle: Arc<AtomicU32>,
    b2r_cache: Arc<Mutex<HashMap<u32, VecDeque<B2RMessage>>>>,
    r2b_cache: Arc<Mutex<HashMap<u32, VecDeque<R2BMessage>>>>,
}

impl B2RServer {
    /// make a new server with a socket path
    /// you need to specify the B2R_SOCKET environment variable when start the bluesim
    pub fn new_with(path: &str) -> Self {
        B2RServer {
            socket_path: path.to_string(),
            cycle: Arc::new(AtomicU32::new(0)),
            b2r_cache: Arc::new(Mutex::new(HashMap::new())),
            r2b_cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Start a thread to run the server.
    /// Create a UnixListener at socket_path.
    /// Return the JoinHandle of that thread.
    /// This function needs to be called before running your Bluesim program.
    /// the server thread returns when bluesim called shut_down_server()
    pub fn serve(&mut self) -> JoinHandle<()> {
        // let probe_infos = self.probe_infos.clone();
        let b2r_cache = self.b2r_cache.clone();
        let r2b_cache = self.r2b_cache.clone();
        let cycle = self.cycle.clone();
        let socket_path = self.socket_path.clone();
        thread::spawn(move || {
            let _ = fs::remove_file(socket_path.as_str());
            let listener = UnixListener::bind(socket_path).expect("Failed to bind Unix listener");
            let mut stream = match listener.incoming().next() {
                Some(stream_res) => stream_res.expect("Fail to connect to bluesim"),
                None => panic!("listener returns a None"),
            };
            loop {
                let message = receive_getput(&mut stream).expect("Fail to deserialize the message");
                match message {
                    GetPutMessage::Get(id) => {
                        // println!("receive get from id {}", id);
                        loop {
                            let mut r2b_cache = r2b_cache.lock().expect("Fail to lock r2b_cache");
                            if let Some(queue) = r2b_cache.get_mut(&id) {
                                if let Some(r2b_message) = queue.pop_front() {
                                    stream
                                        .write_all(&r2b_message.message)
                                        .expect("Fail to write to socket");
                                    drop(r2b_cache);
                                    break;
                                }
                            }
                            drop(r2b_cache);
                        }
                    }
                    GetPutMessage::Put(b2r_message) => {
                        // println!("receive put to id {}", b2r_message.id);
                        // return if reveive message with SHUT_DOWN_ID
                        let id = b2r_message.id;
                        let server_cycle = cycle.load(std::sync::atomic::Ordering::Acquire);
                        if b2r_message.cycles > server_cycle {
                            cycle.store(b2r_message.cycles, std::sync::atomic::Ordering::Release)
                        }
                        let mut b2r_cache = b2r_cache.lock().expect("Fail to lock b2r_cache");
                        let queue = b2r_cache.entry(b2r_message.id).or_default();
                        queue.push_back(b2r_message);
                        if id == SHUT_DOWN_ID {
                            return;
                        }
                    }
                }
            }
        })
    }

    /// Send a message to the probe with ID "id".
    /// Please ensure that message.len() == ceil(get_t_width/8),
    /// where get_t_width is the width of get_t defined in your BSV code.
    pub fn put(&mut self, id: u32, message: Vec<u8>) {
        let r2b_message = R2BMessage { id, message };
        let mut r2b_cache = self.r2b_cache.lock().expect("Fail to lock r2b_cache");
        let queue = r2b_cache.entry(id).or_default();
        queue.push_back(r2b_message);
    }

    /// return the newest message's cycle
    pub fn current_cycle(&self) -> u32 {
        self.cycle.load(std::sync::atomic::Ordering::Acquire)
    }
}

fn get_msg_size(bytes: Vec<u8>) -> u32 {
    u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
}

fn receive_getput(stream: &mut UnixStream) -> Result<GetPutMessage, Box<bincode::ErrorKind>> {
    // println!("connect comeing");
    // the first MSG_SIZE_BYTES bytes is the size of message
    // use to initialize the buffer
    let mut sz_buf: Vec<u8> = vec![0; MSG_SIZE_BYTES];
    stream
        .read_exact(&mut sz_buf)
        .expect("Failed to read from stream");
    let sz_msg: usize = get_msg_size(sz_buf) as usize;
    let mut buffer = vec![0; sz_msg];
    stream
        .read_exact(&mut buffer)
        .expect("Failed to read from stream");
    // println!("sz_msg is {}", sz_msg);
    bincode::deserialize::<GetPutMessage>(&buffer)
}
