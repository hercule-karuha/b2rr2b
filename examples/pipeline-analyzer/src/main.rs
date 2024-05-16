use rb_link::*;

fn main() {
    let mut server = B2RServer::new();
    for i in 0..10 {
        let num: u32 = i;
        server.put(0, num.to_le_bytes().to_vec())
    }
    server.add_probe(1, 32, 32);
    let handlle = server.serve();
    for i in 0..10 {
        let msg = server.get(0);
        match msg {
            Some(msg) => {
                println!(
                    "get from blue id:{}, cycle:{}, data:{}",
                    msg.id,
                    msg.cycles,
                    u32::from_le_bytes([
                        msg.message[0],
                        msg.message[1],
                        msg.message[2],
                        msg.message[3]
                    ])
                );
            }
            None => panic!("fail to get from id:{}", 0),
        }
    }
    let _ = handlle.join();
}
