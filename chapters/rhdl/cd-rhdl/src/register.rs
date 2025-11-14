use crate::prelude::*;

#[derive(Debug,Digital,PartialEq,Default)]
pub struct RegInput<N: BitWidth + Unsigned> {
    pub data_in: Bits<N>,
    pub oe: bool,
    pub we: bool
}