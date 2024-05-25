use super::*;
use std::io::Write;
use std::os::unix::net::UnixStream;
use std::thread;
use std::time::Duration;
#[test]
fn test_get() {
    let mut server = B2RServer::new_with("/tmp/test_get");

    let _ = server.serve();
    thread::sleep(Duration::from_micros(400));
    let data: u64 = 114514;
    let mut stream =
        UnixStream::connect(String::from("/tmp/test_get")).expect("Failed to connect to socket");

    put(114, 514, data.to_le_bytes().to_vec(), &mut stream);

    let mut id_getter = IDGetter::new(&server);

    let msg = id_getter.get(114);
    assert_eq!(msg.id, 114);
    assert_eq!(msg.cycles, 514);
    assert_eq!(msg.message.len(), 8);
    assert_eq!(u64_from_vec(msg.message), 114514);

    let data_vec: Vec<u8> = vec![0x00, 0x11, 0x5c, 0x33, 0x23];
    put(0, 0, data_vec, &mut stream);

    let msg = id_getter.get(0);
    assert_eq!(msg.message[3], 0x33);

    drop(server);
}

#[test]
fn test_try_get() {
    let mut server = B2RServer::new_with("/tmp/test_try_get");

    let _ = server.serve();
    thread::sleep(Duration::from_micros(400));
    let data: u64 = 114514;
    let mut stream = UnixStream::connect(String::from("/tmp/test_try_get"))
        .expect("Failed to connect to socket");
    put(114, 514, data.to_le_bytes().to_vec(), &mut stream);

    let mut id_getter = IDGetter::new(&server);

    let msg = id_getter.try_get(114).unwrap();
    assert_eq!(msg.id, 114);
    assert_eq!(msg.cycles, 514);
    assert_eq!(msg.message.len(), 8);
    assert_eq!(u64_from_vec(msg.message), 114514);

    let msg = id_getter.try_get(114);
    assert!(msg.is_none());

    drop(server);
}

#[test]
fn test_get_cycle() {
    let mut server = B2RServer::new_with("/tmp/test_get_cycle");

    let _ = server.serve();
    thread::sleep(Duration::from_micros(400));

    let data: u64 = 114514;
    let mut stream = UnixStream::connect(String::from("/tmp/test_get_cycle"))
        .expect("Failed to connect to socket");

    put(0, 0, data.to_le_bytes().to_vec(), &mut stream);
    put(0, 1, data.to_le_bytes().to_vec(), &mut stream);
    put(1, 1, data.to_le_bytes().to_vec(), &mut stream);
    put(8, 0, data.to_le_bytes().to_vec(), &mut stream);
    put(3, 0, data.to_le_bytes().to_vec(), &mut stream);
    put(7, 0, data.to_le_bytes().to_vec(), &mut stream);
    put(12, 1, data.to_le_bytes().to_vec(), &mut stream);

    let mut cycle_getter = CycleGetter::new(&server);

    let msgs = cycle_getter.get_cycle_message();
    assert_eq!(msgs.len(), 4);
    drop(server);
}

#[test]
#[should_panic(expected = "Failed to connect to socket")]
fn test_connect_before_server() {
    let mut server = B2RServer::new_with("/tmp/test_connect_before_server");
    let _ = UnixStream::connect(String::from("/tmp/test_connect_before_server"))
        .expect("Failed to connect to socket");
    let _ = server.serve();
}

#[test]
fn test_deserialize_fail() {
    let mut server = B2RServer::new_with("/tmp/test_deserialize_fail");
    let handle = server.serve();
    thread::sleep(Duration::from_micros(400));
    let mut stream = UnixStream::connect(String::from("/tmp/test_deserialize_fail"))
        .expect("Failed to connect to socket");

    let data: u64 = 114514;
    put_data_directly(data.to_le_bytes().to_vec(), &mut stream);
    let join_res = handle.join();
    assert!(join_res.is_err());
}

#[test]
fn test_get_get_pipeline_state() {
    let mut server = B2RServer::new_with("/tmp/test_get_get_pipeline_state");
    let mut pipe_getter = PipeLineGetter::new(&server);

    for i in 0..4 {
        let id: u32 = i;
        pipe_getter.add_fifo_probe(id);
    }
    for i in 10..15 {
        let id: u32 = i;
        pipe_getter.add_rule_probe(id);
    }

    let _ = server.serve();
    thread::sleep(Duration::from_micros(400));
    let mut stream = UnixStream::connect(String::from("/tmp/test_get_get_pipeline_state"))
        .expect("Failed to connect to socket");
    let full_msg = vec![0, 1];
    let empty_msg = vec![1, 0];
    let not_full_empty = vec![1, 1];

    put(0, 0, empty_msg.clone(), &mut stream);
    put(1, 0, not_full_empty.clone(), &mut stream);
    put(2, 0, not_full_empty.clone(), &mut stream);
    put(3, 0, full_msg.clone(), &mut stream);

    put(10, 0, vec![1], &mut stream);
    put(12, 0, vec![1], &mut stream);
    put(14, 0, vec![1], &mut stream);

    thread::sleep(Duration::from_micros(400));
    let state = pipe_getter.get_pipeline_state();

    assert_eq!(state.cycle, 0);
    assert_eq!(state.empty_fifos.len(), 1);
    assert_eq!(state.full_fifos.len(), 1);
    assert_eq!(state.fire_rules.len(), 3);
    assert_eq!(state.empty_fifos[0], 0);
    assert_eq!(state.full_fifos[0], 3);
    assert!(state.fire_rules.contains(&10));
    assert!(state.fire_rules.contains(&12));
    assert!(state.fire_rules.contains(&14));
}

#[test]
#[should_panic]
fn test_probe_type_error() {
    let mut server = B2RServer::new_with("/tmp/test_probe_type_error");
    let mut pipe_getter = PipeLineGetter::new(&server);

    pipe_getter.add_fifo_probe(0);
    pipe_getter.add_rule_probe(1);
    let _ = server.serve();
    thread::sleep(Duration::from_micros(400));
    let mut stream = UnixStream::connect(String::from("/tmp/test_probe_type_error"))
        .expect("Failed to connect to socket");

    put(0, 0, vec![0, 0, 0], &mut stream);

    thread::sleep(Duration::from_micros(400));
    let _state = pipe_getter.get_pipeline_state();
}

#[test]
#[should_panic]
fn test_shut_down() {
    let mut server = B2RServer::new_with("/tmp/test_shut_down");
    let mut pipe_getter = PipeLineGetter::new(&server);

    pipe_getter.add_fifo_probe(0);
    pipe_getter.add_rule_probe(1);
    let _ = server.serve();
    thread::sleep(Duration::from_micros(400));
    let mut stream = UnixStream::connect(String::from("/tmp/test_shut_down"))
        .expect("Failed to connect to socket");

    put(114, 514, vec![0], &mut stream);
    let mut id_getter = IDGetter::new(&server);

    let msg = id_getter.get(114);
    assert_eq!(msg.id, 114);
    put(SHUT_DOWN_ID, 514, vec![0], &mut stream);
    put(114, 514, vec![0], &mut stream);
}

fn u64_from_vec(bytes: Vec<u8>) -> u64 {
    u64::from_le_bytes([
        bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
    ])
}

pub fn put(id: u32, cycles: u32, data: Vec<u8>, stream: &mut UnixStream) {
    // println!("send put");
    thread::sleep(Duration::from_micros(400));
    let b2r_message = B2RMessage {
        id,
        cycles,
        message: data,
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
    thread::sleep(Duration::from_micros(400));
}

pub fn put_data_directly(data: Vec<u8>, stream: &mut UnixStream) {
    // println!("send put");
    thread::sleep(Duration::from_micros(400));

    // The initial 4-byte data specifies the byte count of the message in the u32 format.
    let msg_size = data.len() as MsgSizeType;
    let mut msg_with_size = Vec::with_capacity(MSG_SIZE_BYTES + data.len());
    msg_with_size.extend_from_slice(msg_size.to_le_bytes().as_slice());
    msg_with_size.extend(data.iter());

    stream
        .write_all(&msg_with_size)
        .expect("Failed to write to stream");
    thread::sleep(Duration::from_micros(400));
}
