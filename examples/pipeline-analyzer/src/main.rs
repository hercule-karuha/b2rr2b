use rb_link::*;

fn main() {
    let probe = BlueProbe::new(0, 32, 32);
    for i in 0..10 {
        let num : u32 = i;
        println!("put {} to bluesim", i);
        probe.put(&num.to_le_bytes());
    }
    for i in 0..10 {
        let msg: B2RMessage = probe.get();

        println!(
            "get {} from bluesim",
            u32::from_le_bytes(msg.message.as_slice().try_into().unwrap())
        );
    }

}
