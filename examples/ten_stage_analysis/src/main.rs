use rb_link::*;
use std::thread;
use std::time::Duration;
// a ten stage pipeline analyzer
fn main() {
    let input_data: Vec<u32> = (1..200).collect();

    let mut server = B2RServer::new();

    for i in input_data {
        server.put(20, i.to_le_bytes().to_vec())
    }
    // mark the probe as fifo or fired
    for i in 0..9 {
        server.give_type(ProbeType::Fifo, i as u32);
    }
    for i in 10..19 {
        server.give_type(ProbeType::Fired, i as u32);
    }

    let handlle = server.serve();

    thread::sleep(Duration::from_secs(5));

    let mut fired: u32 = 0;
    loop {
        let state = server.get_pipeline_state();
        if state.fire_rules.len() as u32 > fired {
            fired = state.fire_rules.len() as u32;
        } else if state.empty_fifos.len() + state.full_fifos.len() == 9 { // if all fifos is empty or full the pipeline is stuck
            println!("pipeline stuck at cycle: {}", state.cycle);
            print!("full fifos:");
            for id in state.full_fifos {
                print!(" {} ", id);
            }
            print!("\nempty fifos:");
            for id in state.empty_fifos {
                print!(" {} ", id);
            }
            print!("\nfired rules:");
            for id in state.fire_rules {
                print!(" {} ", id)
            }
            print!("\n");
            break;
        }
    }

    let _ = handlle.join();
}
