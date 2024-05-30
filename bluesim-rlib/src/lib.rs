//! The Rust library called by bluesim.
//! If you want to use RProbe in your bluespec project,
//! please compile this crate into an .a file and then link it to your bluesim executable.
#![warn(clippy::unwrap_used)]
use rb_link::{B2RMessage, GetPutMessage, MsgSizeType, MSG_SIZE_BYTES};
use std::env;
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::sync::OnceLock;

static mut STREAM: OnceLock<UnixStream> = OnceLock::new();

/// # Safety
/// This function should not be called by Rust code.
/// Get data from your rust program.
/// called by RProbe::get_data()
#[no_mangle]
pub unsafe extern "C" fn get(res_ptr: *mut u8, id: u32, _cycles: u32, size: u32) {
    // println!("send get");
    // check the ptr is not null
    if res_ptr.is_null() {
        panic!("res_ptr is a null pointer!");
    }
    if _cycles == u32::MAX {
        panic!("cycles over flow!");
    }
    let mut stream = STREAM.get_or_init(get_stream);

    let get_message = GetPutMessage::Get(id);
    let serialized = bincode::serialize(&get_message).expect("Serialization failed");

    // The initial 4-byte data specifies the byte count of the message in the u32 format.
    let msg_size = serialized.len() as MsgSizeType;
    let mut msg_with_size = Vec::with_capacity(MSG_SIZE_BYTES + serialized.len());
    msg_with_size.extend_from_slice(&msg_size.to_le_bytes());
    msg_with_size.extend(serialized.iter());
    stream
        .write_all(&msg_with_size)
        .expect("Failed to write to stream");

    let res_slice = std::slice::from_raw_parts_mut(res_ptr, size as usize);
    stream
        .read_exact(res_slice)
        .expect("Failed to read from stream");
}

/// # Safety
/// This function should not be called by Rust code.
/// Put data to your rust program.
/// called by RProbe::put_data()
#[no_mangle]
pub unsafe extern "C" fn put(id: u32, cycles: u32, data_ptr: *mut u8, size: u32) {
    // println!("send put");
    // check the ptr is not null
    if data_ptr.is_null() {
        panic!("data_ptr is a null pointer!");
    }
    if cycles == u32::MAX {
        panic!("cycles over flow!");
    }

    let mut stream = STREAM.get_or_init(get_stream);

    let data_slice = std::slice::from_raw_parts(data_ptr, size as usize);
    let b2r_message = B2RMessage {
        id,
        cycles,
        message: data_slice.to_vec(),
    };
    let put_message = GetPutMessage::Put(b2r_message);
    let serialized = bincode::serialize(&put_message).expect("Serialization failed");

    // The initial 4-byte data specifies the byte count of the message in the u32 format.
    let msg_size = serialized.len() as MsgSizeType;
    let mut msg_with_size = Vec::with_capacity(MSG_SIZE_BYTES + serialized.len());
    msg_with_size.extend_from_slice(msg_size.to_le_bytes().as_slice());
    msg_with_size.extend(serialized.iter());

    stream
        .write_all(&msg_with_size)
        .expect("Failed to write to stream");
}

/// # Safety
/// This function should not be called by Rust code.
/// no more message to send,send a shut down message to the server
#[no_mangle]
pub unsafe extern "C" fn shut_down() {
    let mut stream = STREAM.get_or_init(get_stream);

    let put_message = GetPutMessage::ShutDown;
    let serialized = bincode::serialize(&put_message).expect("Serialization failed");

    // The initial 4-byte data specifies the byte count of the message in the u32 format.
    let msg_size = serialized.len() as MsgSizeType;
    let mut msg_with_size = Vec::with_capacity(MSG_SIZE_BYTES + serialized.len());
    msg_with_size.extend_from_slice(msg_size.to_le_bytes().as_slice());
    msg_with_size.extend(serialized.iter());

    stream
        .write_all(&msg_with_size)
        .expect("Failed to write to stream");
}

fn get_stream() -> UnixStream {
    let socket = match env::var("B2R_SOCKET") {
        Ok(path) => path,
        Err(_) => "/tmp/b2rr2b".to_string(),
    };
    UnixStream::connect(socket).expect("Failed to connect to socket")
}
