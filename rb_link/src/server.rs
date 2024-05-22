use crate::config::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::io::{Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;
use std::{fs, usize};

#[derive(Serialize, Deserialize)]
pub enum GetPutMessage {
    Get(u32),
    Put(B2RMessage),
}

#[derive(Serialize, Deserialize)]
struct R2BMessage {
    id: u32,
    message: Vec<u8>,
}

/// Message from Bluesim:
/// - id: ID of the probe that sent the message
/// - cycles: Clock cycles when the message was sent
/// - message: Sent message, where message.len() == ceil(put_t_width/8). put_t_width is the width of put_t defined in your BSV code.
#[derive(Serialize, Deserialize)]
pub struct B2RMessage {
    pub id: u32,
    pub cycles: u32,
    pub message: Vec<u8>,
}

/// A server for interacting with Bluesim.
/// Cache bidirectional data and send data upon receiving requests.
pub struct B2RServer {
    probe_infos: HashMap<ProbeType, Vec<u32>>,
    b2r_cache: Arc<Mutex<HashMap<u32, VecDeque<B2RMessage>>>>,
    r2b_cache: Arc<Mutex<HashMap<u32, VecDeque<R2BMessage>>>>,
}

/// Type of The Probe
/// Fifo: won't get data from rust, sent 2 bytes every cycle, the fist byte is notFull second byte is notEmpty
/// Fired: won't get data from rust, sent 1 bytes message when the rule fired
#[derive(PartialEq, Eq, Hash)]
pub enum ProbeType {
    Fifo,
    Fired,
}

/// The pipeline state
/// cycle: the cycle of the state
/// full_fifos: the ids of the full fifos
/// empty_fifos: the ids of the empty fifos
/// fire_fules: the fired rules
pub struct PipeLineState {
    pub cycle: u32,
    pub full_fifos: Vec<u32>,
    pub empty_fifos: Vec<u32>,
    pub fire_rules: Vec<u32>,
}

impl Default for B2RServer {
    fn default() -> Self {
        Self::new()
    }
}

impl B2RServer {
    /// make a new server
    pub fn new() -> B2RServer {
        B2RServer {
            probe_infos: HashMap::new(),
            b2r_cache: Arc::new(Mutex::new(HashMap::new())),
            r2b_cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    // give a type to a probe so that it can be analysis by server.
    pub fn give_type(&mut self, probe_type: ProbeType, id: u32) {
        let ids = self.probe_infos.entry(probe_type).or_default();
        if !ids.contains(&id) {
            ids.push(id);
        }
    }

    /// Start a thread to run the server.
    /// Create a UnixListener at "/tmp/b2rr2b".
    /// Return the JoinHandle of that thread.
    /// This function needs to be called before running your Bluesim program.
    pub fn serve(&mut self) -> JoinHandle<()> {
        // let probe_infos = self.probe_infos.clone();
        let b2r_cache = self.b2r_cache.clone();
        let r2b_cache = self.r2b_cache.clone();
        thread::spawn(move || {
            let _ = fs::remove_file("/tmp/b2rr2b");
            let listener = UnixListener::bind("/tmp/b2rr2b").expect("Failed to bind Unix listener");
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
                        let mut b2r_cache = b2r_cache.lock().expect("Fail to lock b2r_cache");
                        let queue = b2r_cache.entry(b2r_message.id).or_default();
                        queue.push_back(b2r_message);
                    }
                }
            }
        })
    }

    /// Return the earliest message from the probe with id.
    /// This function will block until there is a message available for retrieval.
    pub fn get(&self, id: u32) -> B2RMessage {
        loop {
            let mut b2r_cache = self.b2r_cache.lock().expect("Fail to lock b2r_cache");
            if let Some(queue) = b2r_cache.get_mut(&id) {
                if let Some(b2r_message) = queue.pop_front() {
                    return b2r_message;
                }
            }
            drop(b2r_cache);
            thread::sleep(Duration::from_millis(100));
        }
    }

    /// Return the earliest message from the probe with id.
    /// This function will return None if there is no message available for retrieval.
    pub fn try_get(&self, id: u32) -> Option<B2RMessage> {
        let mut b2r_cache = self.b2r_cache.lock().expect("Fail to lock b2r_cache");
        if let Some(queue) = b2r_cache.get_mut(&id) {
            if let Some(b2r_message) = queue.pop_front() {
                return Some(b2r_message);
            }
        }
        None
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

    /// Get all the messages sent by the earliest cycle.
    /// If there are no messages available, it will return an empty Vec.
    pub fn get_cycle_message(&mut self) -> Vec<B2RMessage> {
        let mut min_cycles = u32::MAX;
        let mut messages: Vec<B2RMessage> = Vec::new();
        let mut b2r_cache = self.b2r_cache.lock().expect("Fail to lock b2r_cache");
        for queue in b2r_cache.values() {
            if let Some(b2r_message) = queue.front() {
                if b2r_message.cycles < min_cycles {
                    min_cycles = b2r_message.cycles;
                }
            }
        }
        for queue in b2r_cache.values_mut() {
            if let Some(b2r_message) = queue.front() {
                if b2r_message.cycles == min_cycles {
                    messages.push(queue.pop_front().expect("front error"));
                }
            }
        }
        messages
    }

    /// Get all message send by the probe with id.
    pub fn get_id_all(&mut self, id: u32) -> Vec<B2RMessage> {
        let mut b2r_cache = self.b2r_cache.lock().expect("Fail to lock b2r_cache");
        let mut messages: Vec<B2RMessage> = Vec::new();

        if let Some(queue) = b2r_cache.get_mut(&id) {
            while let Some(b2r_message) = queue.pop_front() {
                messages.push(b2r_message);
            }
        }
        messages
    }

    /// Read the earliest cycle messages sent by the probes labeled as "fifo" and "fired", and organize them into a PipeLineState.
    pub fn get_pipeline_state(&mut self) -> PipeLineState {
        let mut state: PipeLineState = PipeLineState {
            cycle: u32::MAX,
            full_fifos: Vec::new(),
            empty_fifos: Vec::new(),
            fire_rules: Vec::new(),
        };
        let fifos: &Vec<u32> = self
            .probe_infos
            .get(&ProbeType::Fifo)
            .expect("No fifo probes!");
        let rules: &Vec<u32> = self
            .probe_infos
            .get(&ProbeType::Fired)
            .expect("No fired probes!");
        let mut b2r_cache: std::sync::MutexGuard<HashMap<u32, VecDeque<B2RMessage>>> =
            self.b2r_cache.lock().expect("Fail to lock b2r_cache");

        for fifo_id in fifos {
            if let Some(messages) = b2r_cache.get(fifo_id) {
                if let Some(first_message) = messages.front() {
                    if first_message.cycles < state.cycle {
                        state.cycle = first_message.cycles;
                    }
                }
            }
        }

        for fifo_id in fifos {
            if let Some(messages) = b2r_cache.get_mut(fifo_id) {
                if let Some(first_message) = messages.front() {
                    if first_message.cycles == state.cycle {
                        // the fifo message len must be 2
                        assert_eq!(first_message.message.len(), 2);
                        let b2r_message = messages.pop_front().expect("front error");
                        if b2r_message.message[0] == 0 {
                            state.full_fifos.push(*fifo_id);
                        } else if b2r_message.message[1] == 0 {
                            state.empty_fifos.push(*fifo_id);
                        }
                    }
                }
            }
        }

        for rule_id in rules {
            if let Some(messages) = b2r_cache.get_mut(rule_id) {
                if let Some(first_message) = messages.front() {
                    if first_message.cycles == state.cycle {
                        // the fifo message len must be 2
                        assert_eq!(first_message.message.len(), 1);
                        let _ = messages.pop_front().expect("front error");
                        state.fire_rules.push(*rule_id);
                    }
                }
            }
        }
        state
    }
}

fn get_msg_size(bytes: Vec<u8>) -> u32 {
    u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
}

fn receive_getput(stream: &mut UnixStream) -> Result<GetPutMessage, Box<bincode::ErrorKind>> {
    // println!("connect comeing");
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
