import "BDPI" function Bit#(n) get(Bit#(32) id, Bit#(32) cycles);
import "BDPI" function Bit#(8) put(Bit#(32) id, Bit#(32) cycles, Bit#(n) data, Bit#(32) size);


interface Probe#(numeric type id, type t);
    method t get_data();
    method Bit#(8) put_data(t data);
endinterface

module mkProbe(Probe#(id, t)) provisos(Bits#(t, n));
    Bit#(32) data_size = fromInteger(valueOf(TDiv#(n,8)));
    Bit#(32) id = fromInteger(valueOf(id));

    Reg#(Bit#(32)) cycles <- mkReg(0);

    rule count;
        cycles <= cycles + 1;
    endrule

    method t get_data();
        Bit#(n) data = get(id, cycles);
        return unpack(data);
    endmethod

    method Bit#(8) put_data(t data);
        let bvec = pack(data);
        return put(id, cycles, bvec, data_size);
    endmethod
endmodule