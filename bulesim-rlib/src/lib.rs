use std::{fs::OpenOptions, io::{Read, Write}};

const B2R_PREFIX: &str = "/tmp/b2r-fifo";
const R2B_PREFIX: &str = "/tmp/r2b-fifo";

#[no_mangle]
pub unsafe extern "C" fn get(res_ptr: *mut u8, id: u32, _cycles: u32, size: u32) {
    let fifo_path = format!("{}{}", R2B_PREFIX, id);
    let mut fifo = match OpenOptions::new().read(true).open(fifo_path.clone()) {
        Ok(f) => f,
        Err(_) => panic!("file to open: {}", fifo_path),
    };

    let mut buffer = vec![0u8; size as usize];
    match fifo.read_exact(&mut buffer) {
        Ok(_) => {}
        Err(err) => panic!("Failed to read from FIFO: {}", err),
    };

    let data_slice = std::slice::from_raw_parts_mut(res_ptr, size as usize);
    data_slice.copy_from_slice(&buffer);
}

#[no_mangle]
pub unsafe extern "C" fn put(id: u32, cycles: u32, data_ptr: *mut u8, size: u32) -> u8 {
    let fifo_path = format!("{}{}", B2R_PREFIX, id);
    let mut fifo = match OpenOptions::new().write(true).open(fifo_path.clone()) {
        Ok(f) => f,
        Err(_) => panic!("Failed to open file: {}", fifo_path),
    };

    let mut buffer = Vec::with_capacity((size + 8) as usize);

    // Write id
    buffer.extend_from_slice(&id.to_le_bytes());

    // Write cycles
    buffer.extend_from_slice(&cycles.to_le_bytes());

    // Write data_ptr data
    let data_slice = std::slice::from_raw_parts(data_ptr, size as usize);
    buffer.extend_from_slice(data_slice);

    match fifo.write_all(&buffer) {
        Ok(_) => 0,
        Err(err) => {
            eprintln!("Failed to write to FIFO: {}", err);
            1
        }
    }
}
