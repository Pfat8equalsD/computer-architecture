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
}

#[derive(Digital, PartialEq, Debug, Default)]
pub enum FetchStage {
    #[default]
    PcToMA,
    MaToMem,
    MemToIr,
    FetchDone
}
use FetchStage::*;

#[derive(Digital, PartialEq, Debug, Default)]
pub enum LoadEaStage {
    // Load EA will always start in this state
    #[default]
    Lea,
    // Load program counter into T1
    Lpc,
    // Increment program counter (assumes in T1)
    Ipc,
    DdispA,
    DdispL,
    // Displacement address in MA
    DispA,
    // Load displacement
    DispL,
    // Load index register
    Ldi,
    Pipd,
    // Load base register
    Ldb,
    // First sum
    Sum1,
    // Final sum
    Sum2,
    // Sentinel state, will get replaced with the next stage after EA load
    EaDone
}
use LoadEaStage::*;
#[derive(Digital, PartialEq, Debug, Default)]
pub enum LoadImmStage {
    #[default]
    ImmLpc,
    ImmIpc,
    ImmA,
    ImmL,
    ImmDone,
}
use LoadImmStage::*;
#[derive(Digital, PartialEq, Debug, Default)]
pub enum LoadTempsStage {
    #[default]
    LoadTs,
    LoadT2Reg,
    LoadT1Reg,
    EAToMA,
    TempsMA,
    TempsML,
    LoadDone,
}
use LoadTempsStage::*;

#[derive(Digital, PartialEq, Debug, Default)]
pub enum State {
    #[default]
    Reset,
    Fetch(FetchStage),
    Decode,
    // Effective addres starting point
    LoadEa(LoadEaStage),
    // Immediate operand starting point
    LoadImm(LoadImmStage),
    // Only register starting point
    LoadTemps(LoadTempsStage),

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

impl ControlUnit {
    pub fn new(init: State) -> Self {
        Self {
            state: DFF::new(init),
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
            FetchDone
        }
        FetchDone => FetchDone
    };
    if next_state == FetchDone {
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
fn extract_base(i: DstOperand) -> Option<BaseRegister> {
    match i {
        DstOperand::BasedAddr(b) => Some(b),
        DstOperand::BasedIndexedAddr(b, _i) => Some(b),
        DstOperand::RegSum(b, _i) => Some(b),
        DstOperand::RegisterAddress(x) => match x {
            AddrRegister::Base(b) => Some(b),
            AddrRegister::Index(_x) => None,
        }
        DstOperand::RegSumIncr(b,_i) => Some(b),
        DstOperand::RegSumDecr(b) => Some(b),
        _ => None
    }
}

#[kernel]
fn extract_index(i: DstOperand) -> Option<IndexRegister> {
    match i {
        DstOperand::IndexedAddr(i) => Some(i),
        DstOperand::BasedIndexedAddr(_b, i) => Some(i),
        DstOperand::RegSum(_b, i) => Some(i),
        DstOperand::RegisterAddress(x) => match x {
            AddrRegister::Base(_x) => None,
            AddrRegister::Index(i) => Some(i),
        }
        DstOperand::RegSumIncr(_b,i) => Some(i),
        DstOperand::RegSumDecr(_b) => Some(IndexRegister::XA),
        _ => None
    }
}

#[kernel]
fn stage_load_ea(s: LoadEaStage, i: DstOperand, mut cs: ControlSignals, is_dst: bool) -> (Option<LoadEaStage>, ControlSignals) {
    let mut invalid = false;
    let state: LoadEaStage = match s {
        Lea => {
            match i {
                DstOperand::DirectAddress => Lpc,
                DstOperand::IndirectAddress => Lpc,
                DstOperand::RegisterAddress(x) => match x {
                    AddrRegister::Base(_x) => Ldb,
                    AddrRegister::Index(_x) => Ldi,
                }
                DstOperand::RegSum(_b, _i) => Ldb,
                DstOperand::RegSumIncr(_b, _i) => Ldb,
                DstOperand::RegSumDecr(_b) => Ldb,
                DstOperand::BasedAddr(_b) => Lpc,
                DstOperand::IndexedAddr(_i) => Lpc,
                DstOperand::BasedIndexedAddr(_b, _i) => Lpc,
                _ => EaDone // Will automatically get translated to invalid
            }
        }
        _ => s
        // This pattern does not work for some whatever reason
        // x => x
    };
    // let state = s;
    let mut displ_t1 = true;
    let after_displ: LoadEaStage = match i {
        // Needs MA = disp
        DstOperand::IndirectAddress => DdispA,
        DstOperand::BasedAddr(_b) => Ldb,
        DstOperand::IndexedAddr(_i) => Ldi,
        DstOperand::BasedIndexedAddr(_b, _i) => Ldb,
        _ => {
            displ_t1 = is_dst;
            EaDone
        }
    };
    let mut ldb_t1 = true;
    let after_ldb = match i {
        DstOperand::RegSum(_b, _x) => Ldi,
        DstOperand::RegSumIncr(_b, _x) => Ldi,
        DstOperand::RegSumDecr(_b) => Ldi,
        DstOperand::BasedAddr(_b) => {
            ldb_t1 = false;
            Sum2
        }
        DstOperand::BasedIndexedAddr(_b, _i) => {
            ldb_t1 = false;
            Sum1
        },
        _ => {
            ldb_t1 = is_dst;
            EaDone
        },
    };
    let mut is_predec = false;
    let mut ldi_t1 = false;
    let after_ldi: LoadEaStage = match i {
        DstOperand::RegSum(_b, _x) => Sum2,
        DstOperand::RegSumIncr(_b, _x) => Pipd,
        DstOperand::RegSumDecr(_b) => {
            is_predec = true;
            Pipd
        },
        DstOperand::IndexedAddr(_i) => Sum2,
        DstOperand::BasedIndexedAddr(_b, _i) => Sum2,  
        _ => {
            ldi_t1 = is_dst;
            EaDone
        }
    };
    let next_state = match state {
        Lpc => {
            cs.t1_we = true;
            cs.pc_oe = true;
            Ipc
        }
        Ipc => {
            cs.t1_oe = true;
            cs.pc_we = true;
            cs.ma_we = true;
            cs.alu_carry = true;
            cs.alu_oe = true;
            cs.alu_sel = ADC;
            DispA
        }
        DispA => {
            cs.ma_oe = true;
            DispL
        }
        DispL => {
            cs.t1_we = displ_t1;
            cs.t2_we = !displ_t1;
            cs.ram_oe = true;
            // Not mandatory for all cases, but needed for indirect addressing
            cs.ma_we = true;
            after_displ
        }
        DdispA => {
            cs.ma_oe = true;
            DdispL
        }
        DdispL => {
            cs.t1_we = is_dst;
            cs.t2_we = !is_dst;
            cs.ram_oe = true;
            EaDone
        }
        Ldi => {
            cs.t1_we = ldi_t1;
            cs.t2_we = !ldi_t1;
            cs.rf_oe = true;
            if let Some(i) = extract_index(i) {
                cs.rf_sel = itr(i);
                after_ldi
            } else {
                invalid = true;
                EaDone
            }
        }
        Ldb => {
            cs.t1_we = ldb_t1;
            cs.t2_we = !ldb_t1;
            cs.rf_oe = true;
            if let Some(i) = extract_base(i) {
                cs.rf_sel = btr(i);
                after_ldb
            } else {
                invalid = true;
                EaDone
            }
        }
        Sum1 => {
            cs.t1_oe = true;
            cs.t2_oe = true;
            cs.t1_we = true;
            cs.alu_oe = true;
            cs.alu_sel = ADC;
            Ldi
        }
        Sum2 => {
            cs.t1_oe = true;
            cs.t2_oe = true;
            cs.t1_we = is_dst;
            cs.t2_we = !is_dst;
            cs.alu_oe = true;
            cs.alu_sel = ADC;
            EaDone
        }
        Pipd => {
            cs.t2_we = is_predec;
            cs.t2_oe = true;
            cs.rf_we = true;
            cs.alu_oe = true;
            cs.alu_sel = if is_predec {SBB2} else {ADC};
            cs.alu_carry = true;
            if let Some(i) = extract_index(i) {
                cs.rf_sel = itr(i);
                Sum2
            } else {
                invalid = true;
                EaDone
            }
        }
        EaDone => {
            invalid = true;
            EaDone
        }
        Lea => {
            invalid = true;
            EaDone
        }


    };

    if invalid {
        (None, cs)
    } else {
        (Some(next_state), cs)
    }
}

#[kernel]
fn stage_load_imm(s: LoadImmStage, mut cs: ControlSignals) -> (State, ControlSignals) {
    let next_state = match s {
        ImmLpc => {
            cs.pc_oe = true;
            cs.t2_we = true;
            ImmIpc
        }
        ImmIpc => {
            cs.ma_we = true;
            cs.t2_oe = true;
            cs.alu_carry = true;
            cs.alu_sel = ADC;
            cs.alu_oe = true;
            cs.pc_we = true;
            ImmA
        }
        ImmA => {
            cs.ma_oe = true;
            ImmL
        }
        ImmL => {
            cs.ram_oe = true;
            cs.t2_we = true;
            ImmDone
        }
        ImmDone => ImmDone
    };
    if next_state == ImmDone {
        (State::LoadTemps(LoadTs), cs)
    } else {
        (State::LoadImm(next_state), cs)
    }
}

#[kernel]
fn stage_load_tmps(s: LoadTempsStage, mut cs: ControlSignals, i: Decoded) -> (State, ControlSignals) {
    let mut invalid = false;
    let mut t2_reg = Reg::dont_care();
    let state = if s == LoadTs {
        match i {
            Decoded::TwoOp { op: _x, src: s, dst: _d} => {
                if let Operand::MaybeDst(x) = s {
                    if let DstOperand::Reg(x) = x {
                        t2_reg = x;
                        LoadT2Reg
                    } else {
                        LoadT1Reg
                    }
                } else {
                    LoadT1Reg
                }
            }
            Decoded::OneOp { op: _x, dst: _d} => LoadT1Reg,
            _ => {
                invalid = true;
                LoadDone
            }
        }
    } else {s};
    let mut t1_reg = Reg::dont_care();
    let state = if state == LoadT1Reg {
        match i {
            Decoded::TwoOp { op: _x, src: _s, dst: d} => {
                if let DstOperand::Reg(x) = d {
                    t1_reg = x;
                    LoadT1Reg
                } else {
                    EAToMA
                }
            }
            Decoded::OneOp { op: _x, dst: d} => {
                if let DstOperand::Reg(x) = d {
                    t1_reg = x;
                    LoadT1Reg
                } else {
                    EAToMA
                }
            }
            _ => {
                invalid = true;
                LoadDone
            }
        }
    } else {state};

    let is_dst = if let Some((_ea, d)) = extract_ea(i) {
        d
    } else {
        true
    };

    let state = if state == EAToMA {
        if let Some((_ea, _d)) = extract_ea(i) {
            EAToMA
        } else {
            LoadDone
        }
    } else {
        state
    };
    let next_state = match state {
        LoadT2Reg => {
            cs.t2_we = true;
            cs.rf_oe = true;
            cs.rf_sel = t2_reg;
            LoadT1Reg
        }
        LoadT1Reg => {
            cs.t1_we = true;
            cs.rf_oe = true;
            cs.rf_sel = t1_reg;
            EAToMA
        }
        EAToMA => {
            cs.t1_oe = is_dst;
            cs.t2_oe = !is_dst;
            cs.alu_oe = true;
            cs.alu_sel = OR;
            cs.ma_we = true;
            match i {
                Decoded::TwoOp { op: _x @ TwoOp::Mov, src: _x2, dst: _x3} => LoadDone,
                Decoded::OneOp { op: _x @ OneOp::MovI, dst: _x2} => LoadDone,
                _ => TempsMA,
            }
        }
        TempsMA => {
            cs.ma_oe = true;
            TempsML
        }
        TempsML => {
            cs.ram_oe = true;
            cs.t1_we = is_dst;
            cs.t2_we = !is_dst;
            LoadDone
        }
        _ => LoadDone
    };

    if invalid {
        (Hlt, cs)
    } else if next_state == LoadDone && s != LoadDone {
        // Delay 1 clock cycle such that the signal for load done is given after
        // every register was written to
        (State::LoadTemps(LoadDone), cs)
    } else if next_state == LoadDone {
        cs.load_done = true;
        (IncPC, cs)
    } else {
        (State::LoadTemps(next_state), cs)
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
        exec_done: false,
        load_done: false,
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
                State::LoadImm(ImmLpc)
            } else {
                State::LoadTemps(LoadTs)
            };
            match d {
                Decoded::TwoOp {op:_x,dst:_x2,src:_x3} => eastate,
                Decoded::OneOp {op:_x,dst:_x2} => eastate,
                Decoded::CfNea(_x @ CfNea::Hlt) => Hlt,
                Decoded::CfNea(_x) => IncPC,
                Decoded::Jcond(_x) => IncPC,
                Decoded::Invalid => Hlt,
            }
        },
        State::LoadEa(l) => {
            if let Some((ea, is_dst)) = extract_ea(d) {
                let state_after_ea = if has_imm(d) {
                    State::LoadImm(ImmLpc)
                } else {
                    State::LoadTemps(LoadTs)
                };

                let (state, cs2) = stage_load_ea(l, ea, cs, is_dst);
                cs = cs2;
                if let Some(ea_state) = state {
                    match ea_state {
                        EaDone => state_after_ea,
                        _ => State::LoadEa(ea_state),
                    }
                } else {
                    Hlt
                }
            } else {
                Hlt
            }
        }
        State::LoadImm(i) => {
            let (state, cs2) = stage_load_imm(i, cs);
            cs = cs2;
            state
        }
        State::LoadTemps(t) => {
            let (state, cs2) = stage_load_tmps(t, cs, d);
            cs = cs2;
            state
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
        // _ => IncPC
        // _ => Reset
    };
    (next_state, cs)
}

#[kernel]
pub fn cu_kernel(_cr: ClockReset, i: (Decoded, AluFlags), q: Q) -> (ControlSignals, D) {
    let (next_state,  cs)  = control_unit(i.0, i.1, q.state);
    (cs, D { state: next_state })
}