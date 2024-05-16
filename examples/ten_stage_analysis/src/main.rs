use rb_link::*;
use std::thread;
use std::time::Duration;

fn main() {
    let mut input_data: Vec<u32> = (1..200).collect();
    input_data[56] = 114514;

    let mut server = B2RServer::new();

    for i in input_data {
        server.put(20, i.to_le_bytes().to_vec())
    }

    let handlle = server.serve();

    thread::sleep(Duration::from_secs(10));

    let mut cycles: u32 = u32::MAX;
    let mut fire_messages: Vec<Vec<B2RMessage>> = Vec::new();
    let mut e_f_messages: Vec<Vec<B2RMessage>> = Vec::new();
    let mut stuck_msg = Vec::new();
    loop {
        let (e_f_msg, f_msg) = divide_message(server.get_cycle_message());
        if cycles == u32::MAX {
            cycles = e_f_msg.first().unwrap().cycles;
            fire_messages.push(f_msg);
            e_f_messages.push(e_f_msg);
        } else if f_msg.len() < fire_messages.last().unwrap().len() {
            cycles = e_f_msg.first().unwrap().cycles;
            println!("pipeline stuck at cycle: {} !!!", cycles);
            e_f_messages.push(e_f_msg);
            stuck_msg = f_msg;
            break;
        } else {
            cycles = e_f_msg.first().unwrap().cycles;
            fire_messages.push(f_msg);
            e_f_messages.push(e_f_msg);
        }
    }
    let before_stuck: &Vec<B2RMessage> = fire_messages.last().unwrap();
    let stuck_ids: Vec<u32> = stuck_msg.iter().map(|msg| msg.id).collect();
    let stuck_msg = before_stuck
        .iter()
        .filter(|msg| !stuck_ids.contains(&msg.id))
        .min_by_key(|msg| msg.message.len());
    match stuck_msg {
        Some(msg) => {
            println!("pipeline stuck at stage:{}", msg.id - 10);
        }
        None => todo!(),
    }

    let _ = handlle.join();
}

fn divide_message(cycle_messages: Vec<B2RMessage>) -> (Vec<B2RMessage>, Vec<B2RMessage>) {
    let mut fire_message: Vec<B2RMessage> = Vec::new();
    let mut e_f_message: Vec<B2RMessage> = Vec::new();
    for msg in cycle_messages {
        if msg.id >= 0 && msg.id <= 8 {
            e_f_message.push(msg);
        } else if msg.id >= 10 && msg.id <= 18 {
            fire_message.push(msg);
        }
    }
    (e_f_message, fire_message)
}
