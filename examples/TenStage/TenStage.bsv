// a ten stage pipeline 
// add 1 to the input in each stage
// will stuck at stage5 if data is 32'h5c

import RProbe::*;
import FIFOF::*;
import Vector::*;

function Bit#(16) gen_e_f(Bool full, Bool empty);
    Bit#(8) full8 = full ? 1 : 0;
    Bit#(8) empty8 = empty ? 1 : 0;
    return {empty8, full8};
endfunction


(* synthesize *)
module mkAdderPipeline(Empty);
    function gen_e_f_probe(Integer x) = mkRProbe(fromInteger(x));
    function gen_fire_probes(Integer x) = mkRProbe(fromInteger(x + 10));

    Vector#(9, FIFOF#(Bit#(32))) fifos <- replicateM(mkSizedFIFOF(5));
    Vector#(9, RProbe#(Bool, Bit#(16))) e_f_probes <- genWithM(gen_e_f_probe);
    Vector#(9, RProbe#(Bool, Bool)) fire_probes <- genWithM(gen_fire_probes);

    RProbe#(Bit#(32), Bit#(32)) recv_probe <- mkRProbe(20);

    rule send_ef;
        e_f_probes[0].put_data(gen_e_f(!fifos[0].notFull(), !fifos[0].notEmpty()));
        e_f_probes[1].put_data(gen_e_f(!fifos[1].notFull(), !fifos[1].notEmpty()));
        e_f_probes[2].put_data(gen_e_f(!fifos[2].notFull(), !fifos[2].notEmpty()));
        e_f_probes[3].put_data(gen_e_f(!fifos[3].notFull(), !fifos[3].notEmpty()));
        e_f_probes[4].put_data(gen_e_f(!fifos[3].notFull(), !fifos[3].notEmpty()));
        e_f_probes[5].put_data(gen_e_f(!fifos[5].notFull(), !fifos[5].notEmpty()));
        e_f_probes[6].put_data(gen_e_f(!fifos[6].notFull(), !fifos[6].notEmpty()));
        e_f_probes[7].put_data(gen_e_f(!fifos[7].notFull(), !fifos[7].notEmpty()));
        e_f_probes[8].put_data(gen_e_f(!fifos[8].notFull(), !fifos[8].notEmpty()));
    endrule

    rule stage1;
        Bit#(32) data = recv_probe.get_data();
        fire_probes[0].put_data(True);
        fifos[0].enq(data);
    endrule

    rule stage2;
        Bit#(32) data = fifos[0].first();
        fifos[0].deq();
        fire_probes[1].put_data(True);
        fifos[1].enq(data + 1);
    endrule

    rule stage3;
        Bit#(32) data = fifos[1].first();
        fifos[1].deq();
        fire_probes[2].put_data(True);
        fifos[2].enq(data + 1);
    endrule

    rule stage4;
        Bit#(32) data = fifos[2].first();
        fifos[2].deq();
        fire_probes[3].put_data(True);
        fifos[3].enq(data + 1);
    endrule
    
    rule stage5;
        Bit#(32) data = fifos[3].first();
        if(data != 32'h5c) begin
            fifos[3].deq();
            fire_probes[4].put_data(True);
            fifos[4].enq(data + 1);
        end
    endrule
    
    rule stage6;
        Bit#(32) data = fifos[4].first();
        fifos[4].deq();
        fire_probes[5].put_data(True);
        fifos[5].enq(data + 1);
    endrule
    
    rule stage7;
        Bit#(32) data = fifos[5].first();
        fifos[5].deq();
        fire_probes[6].put_data(True);
        fifos[6].enq(data + 1);
    endrule
    
    rule stage8;
        Bit#(32) data = fifos[6].first();
        fifos[6].deq();
        fire_probes[7].put_data(True);
        fifos[7].enq(data + 1);
    endrule
    
    rule stage9;    
        Bit#(32) data = fifos[7].first();
        fifos[7].deq();
        fire_probes[8].put_data(True);
        fifos[8].enq(data + 1);
    endrule

    rule stage10;
        Bit#(32) data = fifos[8].first();
        fifos[8].deq();
        $display("data after pipeline: %x", data);
    endrule
endmodule