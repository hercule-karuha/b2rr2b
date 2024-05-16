use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;

#[derive(Serialize, Deserialize)]
pub enum GetPutMessage {
    Get(u32),
    Put(B2RMessage),
}

#[derive(Serialize, Deserialize)]
pub struct B2RMessage {
    pub id: u32,
    pub cycles: u32,
    pub message: Vec<u8>,
}

#[no_mangle]
pub unsafe extern "C" fn get(res_ptr: *mut u8, id: u32, _cycles: u32, size: u32) {
    println!("sned get");
    let res_slice = std::slice::from_raw_parts_mut(res_ptr, size as usize);

    let socket_path = String::from("/tmp/b2rr2b");
    let mut stream = UnixStream::connect(socket_path).expect("Failed to connect to socket");
    let get_message = GetPutMessage::Get(id);

    let serialized = bincode::serialize(&get_message).expect("Serialization failed");
    let msg_size = serialized.len() as u32;

    stream
        .write_all(&msg_size.to_le_bytes())
        .expect("Failed to write to stream");
    stream
        .write_all(&serialized)
        .expect("Failed to write to stream");
    stream
        .read_exact(res_slice)
        .expect("Failed to read from stream");
    if size == 4 {
        println!(
            "get data: {}",
            u32::from_le_bytes([res_slice[0], res_slice[1], res_slice[2], res_slice[3]])
        );
    }
}

#[no_mangle]
pub unsafe extern "C" fn put(id: u32, cycles: u32, data_ptr: *mut u8, size: u32) -> u32 {
    println!("sned put");
    let data_slice = std::slice::from_raw_parts(data_ptr, size as usize);

    let socket_path = String::from("/tmp/b2rr2b");
    let mut stream = UnixStream::connect(socket_path).expect("Failed to connect to socket");
    let b2r_message = B2RMessage {
        id,
        cycles,
        message: data_slice.to_vec(),
    };

    let put_message = GetPutMessage::Put(b2r_message);

    if size == 4 {
        println!(
            "put data: {}",
            u32::from_le_bytes([data_slice[0], data_slice[1], data_slice[2], data_slice[3]])
        );
    }

    let serialized = bincode::serialize(&put_message).expect("Serialization failed");
    let msg_size = serialized.len() as u32;

    stream
        .write_all(&msg_size.to_le_bytes())
        .expect("Failed to write to stream");
    stream
        .write_all(&serialized)
        .expect("Failed to write to stream");
    1
}
