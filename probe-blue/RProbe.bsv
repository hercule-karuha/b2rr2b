typedef 8 BYTE_WIDTH;
typedef 32 WORD_WIDTH;
typedef 16 FIFO_INFO_WIDTH;

import "BDPI" function Bit#(n) get(Bit#(WORD_WIDTH) id, Bit#(WORD_WIDTH) cycles, Bit#(WORD_WIDTH) size);
import "BDPI" function Action put(Bit#(WORD_WIDTH) id, Bit#(WORD_WIDTH) cycles, Bit#(n) data, Bit#(WORD_WIDTH) size);
import FIFOF::*;

interface RProbe#(type get_t, type put_t);
    method get_t get_data();
    method Action put_data(put_t data);
endinterface

module mkRProbe#(Bit#(WORD_WIDTH) id)(RProbe#(get_t, put_t)) provisos(Bits#(get_t, wid_get), Bits#(put_t, wid_put));
    Bit#(WORD_WIDTH) get_size = fromInteger(valueOf(TDiv#(wid_get, BYTE_WIDTH)));
    Bit#(WORD_WIDTH) put_size = fromInteger(valueOf(TDiv#(wid_put, BYTE_WIDTH)));

    Reg#(Bit#(WORD_WIDTH)) cycles <- mkReg(0);

    rule count;
        cycles <= cycles + 1;
    endrule

    method get_t get_data();
        Bit#(n) data = get(id, cycles, get_size);
        return unpack(data);
    endmethod

    method Action put_data(put_t data);
        let bvec = pack(data);
        put(id, cycles, bvec, put_size);
    endmethod
endmodule

module mkFIFOFProbe#(Bit#(WORD_WIDTH) id, FIFOF#(t) fifo)(Empty);
    RProbe#(Bool, Bit#(FIFO_INFO_WIDTH)) e_f_probe <- mkRProbe(id);
    function Bit#(FIFO_INFO_WIDTH) gen_e_f(Bool full, Bool empty);
        Bit#(BYTE_WIDTH) full8 = full ? 1 : 0;
        Bit#(BYTE_WIDTH) empty8 = empty ? 1 : 0;
        return {empty8, full8};
    endfunction
    
    rule send_e_f_info;
        e_f_probe.put_data(gen_e_f(fifo.notFull(), fifo.notEmpty()));
    endrule
endmodule