use crate::prelude::*;
mod alu;
mod decode_unit;
mod prelude;
mod register_file;
mod register;
fn main() {
    let reg = RegFile::<U16>::default();
    let x: Vec<(RegInput<U16>,Reg)> = Vec::new();
    let x = x.with_reset(1).clock_pos_edge(100);
    if let Err(e) = reg.run(x) {
        println!("{}",miette_report(e));
    }
}
