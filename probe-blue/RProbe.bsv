import "BDPI" function Bit#(n) get(Bit#(32) id, Bit#(32) cycles, Bit#(32) size);
import "BDPI" function Action put(Bit#(32) id, Bit#(32) cycles, Bit#(n) data, Bit#(32) size);


interface RProbe#(type get_t, type put_t);
    method get_t get_data();
    method Action put_data(put_t data);
endinterface

module mkRProbe#(Bit#(32) id)(RProbe#(get_t, put_t)) provisos(Bits#(get_t, wid_get), Bits#(put_t, wid_put));
    Bit#(32) get_size = fromInteger(valueOf(TDiv#(wid_get,8)));
    Bit#(32) put_size = fromInteger(valueOf(TDiv#(wid_put,8)));

    Reg#(Bit#(32)) cycles <- mkReg(0);

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