# b2rr2b
## A Bluesim simulation probe framework.



This framework includes the following components:

- `/bluesim-rlib` : A Rust library called by the Bluesim program for receiving and sending data.
- `/probe-blue` : BSV code for the probe, calling Rust functions through the BDPI interface.
- `/rb_link` :A Rust framework for interacting with probes, capable of getting and sending data.



### Uasge

First, you need to write BSV code and instantiate RProbe within it.

```
import RProbe::*;
import FIFOF::*;

(* synthesize *)
module mkAdderPipeline(Empty);
    FIFOF#(Bit#(32)) f2d <- mkFIFOF;
    RProbe#(Bit#(32), Bit#(32)) probe <- mkRProbe(0);

    Reg#(Bit#(32)) fetch_times <- mkReg(0);
    Reg#(Bit#(32)) put_times <- mkReg(0);


    rule doGet if (fetch_times < 10);
        Bit#(32) data = probe.get_data();
        f2d.enq(data);
        fetch_times <= fetch_times + 1;
    endrule

    rule doPut;
        Bit#(32) data = f2d.first;
        f2d.deq;
        probe.put_data(data + 1);
        put_times <= put_times + 1;
        if(put_times == 9) begin
            $finish;
        end
    endrule
endmodule
```

You need to link the Rust library file when linking Bluesim.

```
$ cd bluesim-rilb
$ cargo build
$ cd <your bsv projest path>
// add the .a file after the link command
$ <your link command> ../../bulesim-rlib/target/debug/libblue.a
```

Next, write your Rust code for analyzing the data.

```
fn main() {
    let mut server = B2RServer::new();
    // ensure the data length equal to the defination in bsv
    for i in 0..10 {
        let num: u32 = i;
        server.put(0, num.to_le_bytes().to_vec())
    }

    let handlle = server.serve();

    thread::sleep(Duration::from_secs(3));
	// get data from bluesim
    let msg_vec = server.get_id_all(0);
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
```

