use super::{B2RMessage, B2RServer};
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};

pub struct IDGetter {
    b2r_cache: Arc<Mutex<HashMap<u32, VecDeque<B2RMessage>>>>,
}

impl IDGetter {
    pub fn new(server: &B2RServer) -> Self {
        IDGetter {
            b2r_cache: server.b2r_cache.clone(),
        }
    }
    /// Return the earliest message from the probe with id.
    /// This function will block until there is a message available for retrieval.
    pub fn get(&mut self, id: u32) -> B2RMessage {
        loop {
            let mut b2r_cache = self.b2r_cache.lock().expect("Fail to lock b2r_cache");
            if let Some(queue) = b2r_cache.get_mut(&id) {
                if let Some(b2r_message) = queue.pop_front() {
                    return b2r_message;
                }
            }
            drop(b2r_cache);
        }
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

    /// Return the earliest message from the probe with id.
    /// This function will return None if there is no message available for retrieval.
    pub fn try_get(&mut self, id: u32) -> Option<B2RMessage> {
        let mut b2r_cache = self.b2r_cache.lock().expect("Fail to lock b2r_cache");
        if let Some(queue) = b2r_cache.get_mut(&id) {
            if let Some(b2r_message) = queue.pop_front() {
                return Some(b2r_message);
            }
        }
        None
    }
}

pub struct CycleGetter {
    b2r_cache: Arc<Mutex<HashMap<u32, VecDeque<B2RMessage>>>>,
}

impl CycleGetter {
    pub fn new(server: &B2RServer) -> Self {
        CycleGetter {
            b2r_cache: server.b2r_cache.clone(),
        }
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

pub struct PipeLineGetter {
    fifos: Vec<u32>,
    rules: Vec<u32>,
    b2r_cache: Arc<Mutex<HashMap<u32, VecDeque<B2RMessage>>>>,
}

impl PipeLineGetter {
    pub fn new(server: &B2RServer) -> Self {
        PipeLineGetter {
            fifos: Vec::new(),
            rules: Vec::new(),
            b2r_cache: server.b2r_cache.clone(),
        }
    }

    /// add a fifo probe
    /// The fifo probe won't get data from rust, sent 2 bytes every cycle,
    /// the fist byte is notFull second byte is notEmpty
    pub fn add_fifo_probe(&mut self, id: u32) {
        self.fifos.push(id);
    }

    /// add a rule probe
    /// The rule probe won't get data from rust,
    /// sent 1 bytes message when the rule fired
    pub fn add_rule_probe(&mut self, id: u32) {
        self.rules.push(id);
    }

    /// Read the earliest cycle messages sent by the probes labeled as "fifo" and "fired", and organize them into a PipeLineState.
    pub fn get_pipeline_state(&mut self) -> PipeLineState {
        let mut state: PipeLineState = PipeLineState {
            cycle: u32::MAX,
            full_fifos: Vec::new(),
            empty_fifos: Vec::new(),
            fire_rules: Vec::new(),
        };

        let mut b2r_cache: std::sync::MutexGuard<HashMap<u32, VecDeque<B2RMessage>>> =
            self.b2r_cache.lock().expect("Fail to lock b2r_cache");

        for fifo_id in &self.fifos {
            if let Some(messages) = b2r_cache.get(fifo_id) {
                if let Some(first_message) = messages.front() {
                    if first_message.cycles < state.cycle {
                        state.cycle = first_message.cycles;
                    }
                }
            }
        }

        for fifo_id in &self.fifos {
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

        for rule_id in &self.rules {
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
