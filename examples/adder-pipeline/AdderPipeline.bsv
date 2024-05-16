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