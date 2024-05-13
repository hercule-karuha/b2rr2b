use nix::sys::stat;
use nix::unistd;
use std::{fs::{self, File}, io::{Read, Write}};

const B2R_PREFIX: &str = "/tmp/b2r-fifo";
const R2B_PREFIX: &str = "/tmp/r2b-fifo";

struct BlueProbe {
    id: u32,
    get_t_width: u32,
    put_t_width: u32,
}

struct B2RMessage {
    id: u32,
    cycles: u32,
    message: Vec<u8>,
}

impl BlueProbe {
    fn new(id: u32, get_t_width: u32, put_t_width: u32) -> Self {
        let b2r_fifo_path = format!("{}{}", B2R_PREFIX, id);
        let r2b_fifo_path = format!("{}{}", R2B_PREFIX, id);

        // Create B2R fifo
        if let Err(err) = unistd::mkfifo(b2r_fifo_path.as_str(), stat::Mode::S_IRWXU) {
            println!("Error creating B2R fifo: {}", err);
        }

        // Create R2B fifo
        if let Err(err) = unistd::mkfifo(r2b_fifo_path.as_str(), stat::Mode::S_IRWXU) {
            println!("Error creating R2B fifo: {}", err);
        }

        BlueProbe {
            id,
            get_t_width,
            put_t_width,
        }
    }
    fn get(&self) -> B2RMessage {
        let b2r_fifo_path = format!("{}{}", B2R_PREFIX, self.id);
        let mut file = File::open(&b2r_fifo_path).expect("Error opening B2R fifo");

        let mut buffer = vec![0u8; 8 + (self.get_t_width / 8) as usize];
        file.read_exact(&mut buffer).expect("Error reading from B2R fifo");

        let id = u32::from_le_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]);
        let cycles = u32::from_le_bytes([buffer[4], buffer[5], buffer[6], buffer[7]]);
        let message = buffer[8..].to_vec();

        B2RMessage {
            id,
            cycles,
            message,
        }
    }
    fn put(&self, data: &[u8]) {
        let r2b_fifo_path = format!("{}{}", R2B_PREFIX, self.id);
        let mut file = File::open(&r2b_fifo_path).expect("Error open R2B fifo");

        file.write_all(data).expect("Error writing to R2B fifo");
    }
}

impl Drop for BlueProbe {
    fn drop(&mut self) {
        let b2r_fifo_path = format!("{}{}", B2R_PREFIX, self.id);
        let r2b_fifo_path = format!("{}{}", R2B_PREFIX, self.id);

        // Remove B2R fifo
        if let Err(err) = fs::remove_file(&b2r_fifo_path) {
            println!("Error removing B2R fifo: {}", err);
        }

        // Remove R2B fifo
        if let Err(err) = fs::remove_file(&r2b_fifo_path) {
            println!("Error removing R2B fifo: {}", err);
        }
    }
}
