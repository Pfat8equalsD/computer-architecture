use std::default;

use bitops_rhdl::bitops;
use rhdl::typenum::Diff;

use crate::{prelude::*, register_file::reg};


#[derive(Debug,Digital,PartialEq)]
pub enum Operand {
    MaybeDst(DstOperand),
    Imm,
}

#[derive(Debug,Digital,PartialEq)]
pub enum DstOperand {
    DirectAddress,
    IndirectAddress,
    RegisterAddress(AddrRegister),
    RegSum(BaseRegister,IndexRegister),
    RegSumIncr(BaseRegister,IndexRegister),
    RegSumDecr(BaseRegister),
    BasedAddr(BaseRegister),
    IndexedAddr(IndexRegister),
    BasedIndexedAddr(BaseRegister,IndexRegister),
    Reg(Reg),
}



#[derive(Debug,Digital,PartialEq)]
pub enum AddrRegister{
    Base(BaseRegister),
    Index(IndexRegister),
}

impl Default for AddrRegister {
    fn default() -> Self {
        Self::Base(BaseRegister::default())
    }
}

#[derive(Debug,Digital,PartialEq,Default)]
pub enum BaseRegister{
    #[default]
    BA,
    BB,
}

#[derive(Debug,Digital,PartialEq,Default)]
pub enum IndexRegister{
    #[default]
    XA,
    XB
}


impl Default for Operand {
    fn default() -> Self {
        Self::Imm
    }
}

impl Default for DstOperand {
    fn default() -> Self {
        Self::Reg(Reg::default())
    }
}

#[derive(Debug,Digital,Default,PartialEq)]
pub enum TwoOp {
    #[default]
    Add,
    Adc,
    Sub,
    Sbb,
    And,
    Or,
    Xor,
    Cmp,
    Test,
}

#[bitops]
#[kernel]
fn twop(i: Bits<U3>) -> Option<TwoOp> {
    match i.raw() {
        0b000 => Some(TwoOp::Add),
        0b100 => Some(TwoOp::Adc),
        0b010 => Some(TwoOp::Sub),
        0b110 => Some(TwoOp::Sbb),
        0b001 => Some(TwoOp::And),
        0b101 => Some(TwoOp::Or),
        0b011 => Some(TwoOp::Xor),
        _ => None,
    }
}

#[derive(Debug,Digital,Default,PartialEq)]
pub enum OneOp {
    #[default]
    Mov,
    MovI,
    Push,
    Pop,
    Call,
    Jmp,
    Inc,
    Dec,
    Neg,
    Not,
    Shl,
    Shr,
    Sar,
}

const f:bool = false;
const t:bool = true;

#[bitops]
#[kernel]
fn eacfg(i: Bits<U3>) -> Option<OneOp> {
    match i.raw() {
        0b000 => Some(OneOp::Mov),
        0b010 => Some(OneOp::Push),
        0b110 => Some(OneOp::Pop),
        0b001 => Some(OneOp::Call),
        0b101 => Some(OneOp::Jmp),
        _ => None
    }
}

#[bitops]
#[kernel]
fn oop(i: Bits<U3>) -> Option<OneOp> {
    match i.raw() {
        0b000 => Some(OneOp::Inc),
        0b100 => Some(OneOp::Dec),
        0b010 => Some(OneOp::Neg),
        0b110 => Some(OneOp::Not),
        0b001 => Some(OneOp::Shl),
        0b101 => Some(OneOp::Shr),
        0b011 => Some(OneOp::Sar),
        _ => None
    }
}

#[derive(Debug,Digital,Default,PartialEq)]
/// Control flow instructions with no effective address
/// 
/// Keep in mind that io port addresses are placed on the bus directly,
/// no need to pass them to control unit
pub enum CfNea {
    In,
    Out,
    Pushf,
    Popf,
    Ret,
    Iret,
    #[default]
    Hlt
}

#[bitops]
#[kernel]
fn neacf(i: Bits<U3>) -> Option<CfNea> {
    match i.raw() {
        0b000 => Some(CfNea::In),
        0b100 => Some(CfNea::Out),
        0b010 => Some(CfNea::Pushf),
        0b110 => Some(CfNea::Popf),
        0b001 => Some(CfNea::Ret),
        0b101 => Some(CfNea::Iret),
        0b011 => Some(CfNea::Hlt),
        _ => None
    }
}

#[derive(Debug,Digital,Default,PartialEq)]
/// Conditional Jumps relative to Program Counter
/// 
/// Keep in mind that offsets are placed on the bus directly,
/// no need to pass them to control unit
pub enum Jcond {
    #[default]
    Jbe,
    /// Also JC
    Jb,
    Jle,
    Jl,
    /// Also JZ
    Je,
    Jo,
    Js,
    Jpe,
    Ja,
    /// Also JNC
    Jae,
    Jg,
    Jge,
    /// Also JNZ
    Jne,
    Jno,
    Jns,
    Jpo,
}

#[bitops]
#[kernel]
fn jcond(i: Bits<U4>) -> Jcond {
    match i.raw() {
        0b0000 => Jcond::Jbe,
        0b1000 =>Jcond::Jb,
        0b0100 =>Jcond::Jle,
        0b1100 =>Jcond::Jl,
        0b0010 =>Jcond::Je,
        0b1010 =>Jcond::Jo,
        0b0110 =>Jcond::Js,
        0b1110 =>Jcond::Jpe,
        0b0001 =>Jcond::Ja,
        0b1001 =>Jcond::Jae,
        0b0101 =>Jcond::Jg,
        0b1101 =>Jcond::Jge,
        0b0011 =>Jcond::Jne,
        0b1011 =>Jcond::Jno,
        0b0111 =>Jcond::Jns,
        0b1111 =>Jcond::Jpo,
        _ => Jcond::dont_care()
    }
}

#[derive(Debug,Digital,Default,PartialEq)]
pub enum Decoded{
    TwoOp {op: TwoOp, src: Operand, dst: DstOperand},
    OneOp{op: OneOp, dst: DstOperand},
    CfNea(CfNea),
    Jcond(Jcond),
    #[default]
    Invalid,
}

#[kernel]
fn irx(i: Bits<U1>) -> IndexRegister {
    if i == bits(1) { IndexRegister::XB } else {IndexRegister::XA}
}

#[kernel]
fn brx(i: Bits<U1>) -> BaseRegister {
    if i == bits(1) { BaseRegister::BA } else {BaseRegister::BB}
}

#[bitops]
#[kernel]
/// Decodes the mod and rm parts in one go
fn mod_rm(i: Bits<U16>) -> DstOperand {
    let rm = i[15..13];
    let m = i[9..8];
    match m.raw() {
        0b11 => DstOperand::Reg(reg(rm)),
        0b01 => {
            if rm < bits(0b100) {
                DstOperand::BasedIndexedAddr(brx(rm[1]), irx(rm[0]))
            } else if rm < bits(0b110) {
                DstOperand::IndexedAddr(irx(rm[0]))
            } else {
                DstOperand::BasedAddr(brx(rm[0]))
            }
        }
        0b10 => {
            if rm < bits(0b100) {
                DstOperand::RegSumIncr(brx(rm[1]), irx(rm[0]))
            } else if rm < bits(0b110) {
                DstOperand::RegSumDecr(brx(rm[0]))
            } else if rm[0] == bits(0) {
                DstOperand::DirectAddress
            } else {
                DstOperand::IndirectAddress
            }
        }
        0b00 => {
            if rm < bits(0b100) {
                DstOperand::RegSum(brx(rm[1]), irx(rm[0]))
            } else if rm < bits(0b110) {
                DstOperand::RegisterAddress(AddrRegister::Index(irx(rm[0])))
            } else {
                DstOperand::RegisterAddress(AddrRegister::Base(brx(rm[0])))
            }
        }
        _ => DstOperand::dont_care()
    }
}

#[bitops]
#[kernel]
pub fn decode(i: Bits<U16>) -> Decoded {
    let rm = mod_rm(i);
    let rg = DstOperand::Reg(reg(i[12..10]));
    let (src, dst) = if i[7] == bits(1) {(Operand::MaybeDst(rm), rg)} else {(Operand::MaybeDst(rg), rm)};
    // R3..R2..R1..R0
    match i[3..0].raw() {
        // EA + One op + no imm + control flow
        0b0000 => {
            if let Some(x) = eacfg(i[6..4]){
                return Decoded::OneOp { op: x, dst: rm };
            }
        }
        // EA + One op + no imm + operation
        0b1000 => {
            if let Some(x) = oop(i[6..4]) {
                return Decoded::OneOp { op: x, dst: rm };
            }
        }
        0b0100 => {
            if i[6..4] == bits(0b000) {
                return Decoded::OneOp { op: OneOp::MovI, dst: rm };
            }
        }
        // EA, TwoOp, no imm no save
        0b0010 => {
            if let Some(x) = twop(i[6..4]) {
                match x {
                    TwoOp::Sub => {
                        return Decoded::TwoOp { op: TwoOp::Cmp, src, dst };
                    }
                    TwoOp::And => {
                        return Decoded::TwoOp { op: TwoOp::Test, src, dst };
                    }
                    _ => {}
                }
            }
        }
        0b1010 => {
            if let Some(x) = twop(i[6..4]) {
                return Decoded::TwoOp { op: x, src, dst};
            }
        }
        0b0110 => {
            if let Some(x) = twop(i[6..4]) {
                match x {
                    TwoOp::Sub => {
                        return Decoded::TwoOp { op: TwoOp::Cmp, src: Operand::Imm, dst: rm };
                    }
                    TwoOp::And => {
                        return Decoded::TwoOp { op: TwoOp::Test, src: Operand::Imm, dst: rm };
                    }
                    _ => {}
                }
            }
        }
        0b1110 => {
            if let Some(x) = twop(i[6..4]) {
                return Decoded::TwoOp { op: x, src: Operand::Imm, dst: rm};
            }
        }
        // NEA CF
        0b0001 => {
            if let Some(x) = neacf(i[6..4]) {
                return Decoded::CfNea(x);
            }
        }
        0b1001 => {
            return Decoded::Jcond(jcond(i[7..4]));
        }
        _ => {}
    };
    Decoded::Invalid
}