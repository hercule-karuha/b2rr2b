import "BDPI" function Bit#(n) get(Bit#(32) id, Bit#(32) cycles, Bit#(32) size);
import "BDPI" function Bit#(8) put(Bit#(32) id, Bit#(32) cycles, Bit#(n) data, Bit#(32) size);


interface Probe#(numeric type id, type get_t, type put_t);
    method get_t get_data();
    method Bit#(8) put_data(put_t data);
endinterface

module mkProbe(Probe#(id, get_t, put_t)) provisos(Bits#(get_t, n), Bits#(put_t, m));
    Bit#(32) get_size = fromInteger(valueOf(TDiv#(n,8)));
    Bit#(32) put_size = fromInteger(valueOf(TDiv#(m,8)));
    Bit#(32) id = fromInteger(valueOf(id));

    Reg#(Bit#(32)) cycles <- mkReg(0);

    rule count;
        cycles <= cycles + 1;
    endrule

    method get_t get_data();
        Bit#(n) data = get(id, cycles, get_size);
        return unpack(data);
    endmethod

    method Bit#(8) put_data(put_t data);
        let bvec = pack(data);
        return put(id, cycles, bvec, put_size);
    endmethod
endmodule