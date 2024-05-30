use rb_link::*;
use std::thread;
use std::time::Duration;

fn main() {
    let mut server = B2RServer::new_with("/tmp/adder");
    // the getter used to get data from bluesim
    let mut id_getter = IDGetter::new(&server);
    for i in 0..10 {
        let num: u32 = i;
        server.put(0, num.to_le_bytes().to_vec())
    }

    //start a new thread to receive the data
    let _ = server.serve();

    thread::sleep(Duration::from_secs(5));
 
    let msg_vec = id_getter.get_id_all(0);
    for msg in msg_vec {
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
}
