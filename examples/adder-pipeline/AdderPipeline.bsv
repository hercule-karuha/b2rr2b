import RProbe::*;
import FIFOF::*;

(* synthesize *)
module mkAdderPipeline(Empty);
    FIFOF#(Bit#(32)) f2d <- mkFIFOF;
    RProbe#(0, Bit#(32), Bit#(32)) probe <- mkRProbe;

    rule doFetch;
        $display("get data");
        Bit#(32) data = probe.get_data();
        f2d.enq(data);
    endrule

    rule doDisplay;
        $display("put data");
        Bit#(32) data = f2d.first;
        f2d.deq;
        let put_res = probe.put_data(data + 1);
    endrule
endmodule