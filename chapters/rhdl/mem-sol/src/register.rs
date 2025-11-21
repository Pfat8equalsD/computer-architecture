use rhdl::prelude::*;
use rhdl_fpga::{core::dff::*, reset::negating_conditioner::NegatingConditioner};
pub mod fixture;
//  register
#[derive(Digital, PartialEq, Eq)]
pub struct RegisterInput {
    pub oe: Bits<U1>,
    pub we: Bits<U1>,
    pub data_in: Bits<U16>
}

#[derive(Synchronous,SynchronousDQ,Clone,Debug)]
pub struct Register {
    pub memory: DFF<Bits<U16>>
}

impl Default for Register {
    fn default() -> Self {
        Self{ 
            memory: DFF::new(Bits::<U16>::default())
       }
    }
}

impl SynchronousIO for Register {
    type I = RegisterInput;
    type O = Bits<U16>;
    type Kernel = register_kernel;
}

#[kernel]
pub fn register_kernel (_cr: ClockReset, _i: RegisterInput, _q: Q) -> (Bits<U16>, D) {

    // (bits(0), D { memory: bits(0)})
    (if _i.oe == bits(1) && _i.we == bits(0) {_q.memory} else {bits(0)}, D { memory: if _i.we == bits(1) {_i.data_in} else {_q.memory} })
}


use crate::doc::write_svg;

pub fn sim_reg() -> Result<(), RHDLError> {
    let reg = Register::default();

    let input = vec! [
        RegisterInput {
            oe: bits(0),
            we: bits(1),
            data_in: bits(15),
        },
        RegisterInput {
            oe: bits(1),
            we: bits(0),
            data_in: bits(15),
        },
        RegisterInput {
            oe: bits(0),
            we: bits(0),
            data_in: bits(15),
        },

    ];

    let input_data = input.into_iter().with_reset(1).clock_pos_edge(100);
    let vcd = reg.run(input_data)?.collect::<Vcd>();

    let _ = write_svg(vcd, "reg.svg");

    type T = Adapter<Register, Red>;
    let top2: T = Adapter::new(Register::default());
    let mut fixture = Fixture::new("top2", top2);
    fixture.add_driver(rhdl_bsp::ok::drivers::xem7010::sys_clock::sys_clock(
        &path!(.clock_reset.val().clock)
    )?);
    fixture.
    println!("{}", vp);
    // vcd.dump_to_file("ram.vcd")?;

    Ok(())
}
