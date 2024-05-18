use super::*;
use std::thread;
use std::time::Duration;

#[test]
fn test_get() {
    let mut server = B2RServer::new();

    let _ = server.serve();
    let data: u64 = 114514;

    put(114, 514, data.to_le_bytes().to_vec());

    let msg = server.get(114);
    assert_eq!(msg.id, 114);
    assert_eq!(msg.cycles, 514);
    assert_eq!(msg.message.len(), 8);
    assert_eq!(u64_from_vec(msg.message), 114514);

    let data_vec: Vec<u8> = vec![0x00, 0x11, 0x5c, 0x33, 0x23];
    put(0, 0, data_vec);

    let msg = server.get(0);
    assert_eq!(msg.message[3], 0x33);
    drop(server);
}

#[test]
fn test_try_get() {
    let mut server = B2RServer::new();

    let _ = server.serve();
    let data: u64 = 114514;

    put(114, 514, data.to_le_bytes().to_vec());

    let msg = server.try_get(114).unwrap();
    assert_eq!(msg.id, 114);
    assert_eq!(msg.cycles, 514);
    assert_eq!(msg.message.len(), 8);
    assert_eq!(u64_from_vec(msg.message), 114514);

    let msg = server.try_get(114);
    assert!(msg.is_none());

    drop(server);
}

#[test]
fn test_get_cycle() {
    let mut server = B2RServer::new();

    let _ = server.serve();
    let data: u64 = 114514;

    put(0, 0, data.to_le_bytes().to_vec());
    put(0, 1, data.to_le_bytes().to_vec());
    put(1, 1, data.to_le_bytes().to_vec());
    put(8, 0, data.to_le_bytes().to_vec());
    put(3, 0, data.to_le_bytes().to_vec());
    put(7, 0, data.to_le_bytes().to_vec());
    put(12, 1, data.to_le_bytes().to_vec());

    let msgs = server.get_cycle_message();
    assert_eq!(msgs.len(), 4);
    drop(server);
}

fn u64_from_vec(bytes: Vec<u8>) -> u64 {
    u64::from_le_bytes([
        bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
    ])
}

pub fn put(id: u32, cycles: u32, data: Vec<u8>) {
    // println!("send put");
    thread::sleep(Duration::from_micros(100));
    let socket_path = String::from("/tmp/b2rr2b");
    let mut stream = UnixStream::connect(socket_path).expect("Failed to connect to socket");
    let b2r_message = B2RMessage {
        id,
        cycles,
        message: data,
    };

    let put_message = GetPutMessage::Put(b2r_message);

    let serialized = bincode::serialize(&put_message).expect("Serialization failed");

    // The initial 4-byte data specifies the byte count of the message in the u32 format.
    let msg_size = serialized.len() as u32;
    stream
        .write_all(&msg_size.to_le_bytes())
        .expect("Failed to write to stream");
    stream
        .write_all(&serialized)
        .expect("Failed to write to stream");
    thread::sleep(Duration::from_micros(100));
}
