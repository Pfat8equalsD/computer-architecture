use crate::prelude::*;
use rhdl::prelude::*;
use rhdl_fpga::core::dff::DFF;
#[allow(non_camel_case_types)]
#[derive(Debug, Digital, Default, PartialEq)]
pub enum Reg {
    #[default]
    RA,
    RB,
    RC,
    SP,
    XA,
    XB,
    BA,
    BB,
}

#[kernel]
pub fn reg(i: Bits<U3>) -> Reg {
    match i.raw() {
        0 => RA,
        1 => RB,
        2 => RC,
        3 => SP,
        4 => XA,
        5 => XB,
        6 => BA,
        7 => BB,
        _ => RA, // Never reached
    }
}

#[kernel]
pub fn rtb(i: Reg) -> Bits<U3> {
    bits(match i {
        RA => 0,
        RB => 1,
        RC => 2,
        SP => 3,
        XA => 4,
        XB => 5,
        BA => 6,
        BB => 7,
    })
}

pub struct RegFile<N: Digital> {
    rg: [DFF<Bits<N>>; 8]
}

impl<N: BitWidth + Unsigned> Default for RegFile<N> {
    fn default() -> Self {
        Self { rg: core::array::from_fn(|_| DFF::new(Bits::default())) }
    }
}

#[kernel]
pub fn reg_file(_cr: ClockReset, 
