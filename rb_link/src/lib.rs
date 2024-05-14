use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::io::{Read, Write};
use std::os::unix::net::UnixListener;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

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
    pub fn serve(&mut self) {
        let probe_infos = self.probe_infos.clone();
        let b2r_cache = self.b2r_cache.clone();
        let r2b_cache = self.r2b_cache.clone();
        thread::spawn(move || {
            let listener = UnixListener::bind("/tmp/b2rr2b").expect("Failed to bind Unix listener");
            for stream in listener.incoming() {
                match stream {
                    Ok(mut stream) => {
                        let mut buffer = Vec::new();
                        stream
                            .read_to_end(&mut buffer)
                            .expect("Failed to read from stream");
                        if let Ok(message) = bincode::deserialize::<GetPutMessage>(&buffer) {
                            match message {
                                GetPutMessage::Get(id) => loop {
                                    let mut r2b_cache = r2b_cache.lock().unwrap();
                                    if let Some(queue) = r2b_cache.get_mut(&id) {
                                        if let Some(b2r_message) = queue.pop_front() {
                                            let serialized =
                                                bincode::serialize(&b2r_message).unwrap();
                                            stream.write_all(&serialized).unwrap();
                                            drop(r2b_cache);
                                            break;
                                        }
                                    }
                                    drop(r2b_cache);
                                    thread::sleep(Duration::from_millis(100));
                                },
                                GetPutMessage::Put(b2r_message) => {
                                    let mut b2r_cache = b2r_cache.lock().unwrap();
                                    let queue = b2r_cache
                                        .entry(b2r_message.id)
                                        .or_insert_with(VecDeque::new);
                                    queue.push_back(b2r_message);
                                }
                            }
                        } else {
                            eprintln!("Failed to deserialize GetPutMessage");
                        }
                    }
                    Err(err) => {
                        eprintln!("Error accepting connection: {}", err);
                    }
                }
            }
        });
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

    pub fn put(&mut self, id: u32, message: Vec<u8>) {
        let r2b_message = R2BMessage { id, message };
        let mut r2b_cache = self.r2b_cache.lock().unwrap();
        let queue = r2b_cache.entry(id).or_insert_with(VecDeque::new);
        queue.push_back(r2b_message);
    }
}
