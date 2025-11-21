use rhdl::prelude::*;
use rhdl_fpga::reset::negating_conditioner::NegatingConditioner;

use crate::register::{Register, RegisterInput};

#[derive(Clone, Circuit, CircuitDQ, Default)]
pub struct U<
    W: Domain, // Clock domain for reset signal
    R: Domain, // Clock domain for everything else
> {
    pub resetn_conditioner: NegatingConditioner<W, R>,
    pub memory: Adapter<Register, R>
}

#[derive(PartialEq, Digital, Timed)]
pub struct I<W: Domain, R: Domain> {
    pub reset_n: Signal<ResetN, W>,
    pub clock: Signal<Clock, R>,
    pub reg: Signal<RegisterInput, R>,
}

#[derive(PartialEq, Digital, Timed)]
pub struct O<R: Domain> {
    pub reg: Signal<Bits<U16>, R>,
}

impl<W: Domain, R: Domain> CircuitIO for U<W, R> {
    type I = I<W, R>;
    type O = O<R>;
    type Kernel = fixture_kernel<W, R>;
}

#[kernel]
pub fn fixture_kernel<W: Domain, R: Domain>(i: I<W, R>, q: Q<W, R>) -> (O<R>, D<W, R>) {
    let mut d = D::<W, R>::dont_care();
    let mut o = O::<R>::dont_care();
    // Connect the reset conditioner
    d.resetn_conditioner.reset_n = i.reset_n;
    d.resetn_conditioner;
    d.resetn_conditioner.clock = i.clock;
    // Connect the register
    d.memory.clock_reset = signal(ClockReset {
        clock: i.clock.val(),
        reset: q.resetn_conditioner.val(),
    });
    // Connect the register's axi bus inputs to the fixture's axi bus input
    d.memory.input = signal(i.reg.val());
    // Connect the axi bus output signals
    o.reg = signal(q.memory.val());
    (o, d)
}

pub fn fixture_main() -> Result<(), RHDLError> {
    let uut = U::<Red,Blue>::default();
    let mut top = Fixture::new("reg_top", uut);
    // top.add_driver(rhdl_bsp::);
    bind!(top, oe -> input.reg.val().oe);
    bind!(top, we -> input.reg.val().we);
    bind!(top, data_in -> input.reg.val().data_in);
    bind!(top, clk -> input.clock.val());
    bind!(top, rstn -> input.reset_n.val());
    bind!(top, out -> output.reg.val());
    // bind!(top, we -> input.val().we);
    // bind!(top, data_in -> input.val().data_in);
    let vlog = top.module()?;
    let vp = vlog.pretty();
    println!("{}", vp);
    Ok(())
}