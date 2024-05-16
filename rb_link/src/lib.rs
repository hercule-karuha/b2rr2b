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

#[allow(dead_code)]
#[allow(unused_variables)]
pub struct ProbeInfo {
    id: u32,
    get_t_width: u32,
    put_t_width: u32,
}

#[derive(Serialize, Deserialize)]
struct R2BMessage {
    id: u32,
    message: Vec<u8>,
}

#[allow(dead_code)]
#[allow(unused_variables)]
#[derive(Serialize, Deserialize)]
pub struct B2RMessage {
    pub id: u32,
    pub cycles: u32,
    pub message: Vec<u8>,
}

#[allow(dead_code)]
#[allow(unused_variables)]
pub struct B2RServer {
    probe_infos: Arc<Mutex<HashMap<u32, ProbeInfo>>>,
    b2r_cache: Arc<Mutex<HashMap<u32, VecDeque<B2RMessage>>>>,
    r2b_cache: Arc<Mutex<HashMap<u32, VecDeque<R2BMessage>>>>,
}

impl B2RServer {
    pub fn new() -> B2RServer {
        B2RServer {
            probe_infos: Arc::new(Mutex::new(HashMap::new())),
            b2r_cache: Arc::new(Mutex::new(HashMap::new())),
            r2b_cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    pub fn add_probe(&mut self, id: u32, get_t_width: u32, put_t_width: u32) {
        let probe_info = ProbeInfo {
            id,
            get_t_width,
            put_t_width,
        };
        self.probe_infos.lock().unwrap().insert(id, probe_info);
    }
    pub fn serve(&mut self) -> JoinHandle<()> {
        let _probe_infos = self.probe_infos.clone();
        let b2r_cache = self.b2r_cache.clone();
        let r2b_cache = self.r2b_cache.clone();
        thread::spawn(move || {
            let _ = fs::remove_file("/tmp/b2rr2b");
            let listener = UnixListener::bind("/tmp/b2rr2b").expect("Failed to bind Unix listener");
            for stream in listener.incoming() {
                let mut stream = match stream {
                    Ok(stream) => stream,
                    Err(_) => todo!(),
                };
                let message = match receive_getput(&mut stream) {
                    Ok(m) => m,
                    Err(_) => todo!(),
                };
                match message {
                    GetPutMessage::Get(id) => {
                        println!("receive get from id {}", id);
                        loop {
                            let mut r2b_cache = r2b_cache.lock().unwrap();
                            if let Some(queue) = r2b_cache.get_mut(&id) {
                                if let Some(r2b_message) = queue.pop_front() {
                                    stream.write_all(&r2b_message.message).unwrap();
                                    drop(r2b_cache);
                                    break;
                                }
                            }
                            drop(r2b_cache);
                            thread::sleep(Duration::from_millis(100));
                        }
                    }
                    GetPutMessage::Put(b2r_message) => {
                        println!("receive put to id {}", b2r_message.id);
                        let mut b2r_cache = b2r_cache.lock().unwrap();
                        let queue = b2r_cache
                            .entry(b2r_message.id)
                            .or_insert_with(VecDeque::new);
                        queue.push_back(b2r_message);
                    }
                }
            }
        })
    }

    pub fn get(&self, id: u32) -> Option<B2RMessage> {
        loop {
            let mut b2r_cache = self.b2r_cache.lock().unwrap();
            if let Some(queue) = b2r_cache.get_mut(&id) {
                if let Some(b2r_message) = queue.pop_front() {
                    return Some(b2r_message);
                }
            }
            drop(b2r_cache);
            thread::sleep(Duration::from_millis(100));
        }
    }

    pub fn try_get(&self, id: u32) -> Option<B2RMessage> {
        let mut b2r_cache = self.b2r_cache.lock().unwrap();
        if let Some(queue) = b2r_cache.get_mut(&id) {
            if let Some(b2r_message) = queue.pop_front() {
                return Some(b2r_message);
            }
        }
        None
    }

    pub fn put(&mut self, id: u32, message: Vec<u8>) {
        let r2b_message = R2BMessage { id, message };
        let mut r2b_cache = self.r2b_cache.lock().unwrap();
        let queue = r2b_cache.entry(id).or_insert_with(VecDeque::new);
        queue.push_back(r2b_message);
    }

    pub fn get_cycle_message(&mut self) -> Vec<B2RMessage> {
        let mut min_cycles = u32::MAX;
        let mut messages: Vec<B2RMessage> = Vec::new();
        let mut b2r_cache = self.b2r_cache.lock().unwrap();
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
                    messages.push(queue.pop_front().unwrap());
                }
            }
        }
        messages
    }

    pub fn get_id_all(&mut self, id: u32) -> Vec<B2RMessage> {
        let mut b2r_cache = self.b2r_cache.lock().unwrap();
        let mut messages: Vec<B2RMessage> = Vec::new();

        if let Some(queue) = b2r_cache.get_mut(&id) {
            while let Some(b2r_message) = queue.pop_front() {
                messages.push(b2r_message);
            }
        }
        messages
    }
}

fn get_msg_size(bytes: Vec<u8>) -> u32 {
    u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
}

fn receive_getput(stream: &mut UnixStream) -> Result<GetPutMessage, Box<bincode::ErrorKind>> {
    // println!("connect comeing");
    let mut sz_buf: Vec<u8> = vec![0; 4];
    stream
        .read_exact(&mut sz_buf)
        .expect("Failed to read from stream");
    let sz_msg: usize = get_msg_size(sz_buf) as usize;
    let mut buffer = vec![0; sz_msg];
    let _ = stream
        .read_exact(&mut buffer)
        .expect("Failed to read from stream");
    // println!("sz_msg is {}", sz_msg);
    return bincode::deserialize::<GetPutMessage>(&buffer);
}