use crate::{decode_unit::{Decoded, DstOperand, Operand, TwoOp}, prelude::*};

#[derive(Digital, PartialEq, Debug)]
pub struct ControlSignals {
    pub rf_sel: Reg,
    pub rf_oe: bool,
    pub rf_we: bool,
    pub t1_oe: bool,
    pub t1_we: bool,
    pub t2_oe: bool,
    pub t2_we: bool,
    pub ma_oe: bool,
    pub ma_we: bool,
    pub ram_oe: bool,
    pub ram_we: bool,
    pub alu_carry: bool,
    pub alu_oe: bool,
    pub alu_sel: AluOp,
    pub pc_we: bool,
    pub pc_oe: bool,
    pub ir_we: bool,
    pub ir_oe: bool,
    pub fr_we: bool,
    pub fr_oe: bool,
    pub fr_sel_bus: bool,
    pub load_done: bool,
    pub exec_done: bool,
    pub instruction_done: bool, // State == FetchStage(PcToMa)?
}

#[derive(Digital, PartialEq, Debug, Default)]
pub enum FetchStage {
    #[default]
    PcToMA,
    MaToMem,
    MemToIr
}
use FetchStage::*;

#[derive(Digital, PartialEq, Debug, Default)]
pub enum LoadEaStage {
    #[default]
    Lea,
    Ipc,
    MemW,
    MemL,
    Ldi,
    Rs,
}
use LoadEaStage::*;

#[derive(Digital, PartialEq, Debug, Default)]
pub enum State {
    #[default]
    Reset,
    Fetch(FetchStage),
    Decode,
    // Effective addres starting point
    LoadEa(LoadEaStage),
    // Immediate operand starting point
    LoadImm,
    // Only register starting point
    LoadRegs,

    LoadCf,
    LoadJc,
    IncPC,
    IncPC1,
    Hlt,
}

use State::*;

#[derive(Synchronous, SynchronousDQ, Clone, Debug)]
pub struct ControlUnit {
    state: DFF<State>,
}

impl Default for ControlUnit {
    fn default() -> Self {
        Self {
            state: DFF::new(State::default()),
        }
    }
}

impl SynchronousIO for ControlUnit {
    type I = (Decoded, AluFlags);
    type O = ControlSignals;
    type Kernel = cu_kernel;
}

#[kernel]
fn stage_fetch(s: FetchStage, mut cs: ControlSignals) -> (State, ControlSignals) {
    let next_state = match s {
        PcToMA => {
            cs.pc_oe = true;
            cs.ma_we = true;
            MaToMem
        }
        MaToMem => {
            cs.ma_oe = true;
            MemToIr
        }
        MemToIr => {
            cs.ram_oe = true;
            cs.ir_we = true;
            PcToMA
        }
    };
    if next_state == PcToMA {
        (Decode, cs)
    } else {
        (State::Fetch(next_state), cs)
    }
}

// #[kernel]
// fn has_imm(i: Decoded) -> bool {

// }

#[kernel]
fn has_imm(i: Decoded) -> bool {
    if let Decoded::TwoOp { op: _x, src: _x3 @ Operand::Imm , dst: _x2 } = i {
        true
    } else if let Decoded::OneOp {op: _x1 @ OneOp::MovI, dst: _x} = i {
        true
    } else {
        false
    }
}

#[kernel]
fn extract_ea(i: Decoded) -> Option<(DstOperand, bool)> {
    match i {
        Decoded::TwoOp { op: _x, src: _x3 @ Operand::Imm, dst} => {
            if let DstOperand::Reg(_r1) = dst {
                None
            } else {
                Some((dst, true))
            }
        }
        Decoded::TwoOp { op: _x, src, dst } => {
            // Imm covered by arm above
            if let Operand::MaybeDst(src) = src {
                if let DstOperand::Reg(_r) = src {
                    if let DstOperand::Reg(_r1) = dst {
                        None
                    } else {
                        Some((dst,true))
                    }
                } else {
                    Some((src,false))
                }
            } else {None}
        }
        Decoded::OneOp { op: _x, dst } => {
            if let DstOperand::Reg(_r1) = dst {
                None
            } else {
                Some((dst,true))
            }
        }
        _ => None
    }
}


#[kernel]
fn stage_load_ea(s: LoadEaStage, i: DstOperand, mut cs: ControlSignals, is_dst: bool) -> (State, ControlSignals) {
    let stage_after_ea = Hlt;
    let mut invalid = false;
    let next_state = match i {
        DstOperand::DirectAddress => {
            match s {
                Lea => {
                    cs.pc_oe = true;
                    cs.t1_we = true;
                    Ipc
                }
                Ipc => {
                    cs.pc_we = true;
                    cs.t1_oe = true;
                    cs.alu_carry = true;
                    cs.alu_sel = ADC;
                    cs.alu_oe = true;
                    if is_dst {cs.t1_we = true} else {cs.t2_we = true};
                    Lea
                }
                _ => {
                    invalid = true;
                    Lea
                }
            }
        }
        DstOperand::IndirectAddress => {
            match s {
                Lea => {
                    cs.pc_oe = true;
                    cs.t1_we = true;
                    Ipc
                }
                Ipc => {
                    cs.pc_we = true;
                    cs.ma_we = true;
                    cs.t1_oe = true;
                    cs.alu_carry = true;
                    cs.alu_sel = ADC;
                    cs.alu_oe = true;
                    MemW
                }
                MemW => {
                    cs.ma_oe = true;
                    MemL
                }
                MemL => {
                    cs.ram_oe = true;
                    if is_dst {cs.t1_we = true} else {cs.t2_we = true};
                    Lea
                }
                _ => {
                    invalid = true;
                    Lea
                }
            }
        }
        DstOperand::RegSum(b, i) => {
            match s {
                Lea => {
                    cs.t1_we = true;
                    cs.rf_sel = btr(b);
                    cs.rf_oe = true;
                    Ldi
                }
                Ldi => {
                    cs.t2_we = true;
                    cs.rf_sel = itr(i);
                    cs.rf_oe = true;
                    Rs
                }
                Rs => {
                    if is_dst {cs.t1_we = true} else {cs.t2_we = true};
                    Lea
                }
                _ => {
                    invalid = true;
                    Lea
                }
            }
        }
    
        _ => {
            invalid = true;
            Lea
        }
    };
    if invalid {
        (Hlt, cs)
    } else if next_state == Lea {
        cs.load_done = true;
        (stage_after_ea,cs)
    } else {
        (State::LoadEa(next_state), cs)
    }
}

#[kernel]
pub fn control_unit(d: Decoded, _f: AluFlags, s: State) -> (State, ControlSignals) {
    let mut cs = ControlSignals {
        rf_sel: RA,
        rf_oe: false,
        rf_we: false,
        t1_oe: false,
        t1_we: false,
        t2_oe: false,
        t2_we: false,
        ma_oe: false,
        ma_we: false,
        ram_oe: false,
        ram_we: false,
        alu_carry: false,
        alu_oe: false,
        alu_sel: ADC,
        pc_we: false,
        pc_oe: false,
        ir_we: false,
        ir_oe: false,
        fr_we: false,
        fr_oe: false,
        fr_sel_bus: false,
    };

    let next_state = match s {
        State::Reset => State::Fetch(PcToMA),
        State::Fetch(f) => {
            let (state, cs2) = stage_fetch(f, cs);
            cs = cs2;
            state
        },
        Decode => {
            let eastate = if let Some(_x) = extract_ea(d) {
                State::LoadEa(Lea)
            } else if has_imm(d) {
                LoadImm
            } else {
                LoadRegs
            };
            match d {
                Decoded::TwoOp {op:_x,dst:_x2,src:_x3} => eastate,
                Decoded::OneOp {op:_x,dst:_x2} => eastate,
                Decoded::CfNea(_x) => LoadCf,
                Decoded::Jcond(_x) => LoadJc,
                Decoded::Invalid => Hlt,
            }
        },
        State::LoadEa(l) => {
            if let Some((ea, is_dst)) = extract_ea(d) {
                let (state, cs2) = stage_load_ea(l, ea, cs, is_dst);
                cs = cs2;
                state
            } else {
                Hlt
            }
        }
        IncPC => {
            cs.pc_oe = true;
            cs.t1_we = true;
            IncPC1
        },
        IncPC1 => {
            cs.alu_carry = true;
            cs.alu_oe = true;
            cs.alu_sel = ADC;
            cs.t1_oe = true;
            cs.pc_we = true;
            State::Fetch(PcToMA)
        }
        Hlt => Hlt,
        _ => Reset
    };
    (next_state, cs)
}

#[kernel]
pub fn cu_kernel(_cr: ClockReset, i: (Decoded, AluFlags), q: Q) -> (ControlSignals, D) {
    let (next_state,  cs)  = control_unit(i.0, i.1, q.state);
    (cs, D { state: next_state })
}