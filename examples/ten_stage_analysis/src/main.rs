use rb_link::*;
use std::thread;
use std::time::Duration;
// a ten stage pipeline analyzer
fn main() {
    let input_data: Vec<u32> = (1..200).collect();

    let mut server = B2RServer::new_with("/tmp/ten_stage");
    let mut pipe_getter = PipeLineGetter::new(&server);

    // marked probes
    for i in 0..9 {
        let id: u32 = i;
        pipe_getter.add_fifo_probe(id);
    }
    for i in 10..19 {
        let id: u32 = i;
        pipe_getter.add_rule_probe(id);
    }

    // set input
    for i in input_data {
        server.put(20, i.to_le_bytes().to_vec())
    }

    let handlle = server.serve();

    thread::sleep(Duration::from_secs(5));

    let mut _fired: u32 = 0;
    loop {
        let state = pipe_getter.get_pipeline_state();
        // if rules fired in cycle i+1 > if rules fired in cycle i the pipeline is not stuck
        if state.fire_rules.len() as u32 > _fired {
            _fired = state.fire_rules.len() as u32;
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
                print!(" {}  ", id)
            }
            print!("\n");
            break;
        }
    }

    let _ = handlle.join();
}
