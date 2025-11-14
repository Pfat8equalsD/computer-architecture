pub use crate::alu::{AluInput,AluOutput,AluFlags,AluOp::{self,*}};
pub use crate::register_file::Reg::{self,*};
pub use rhdl::prelude::*;
pub use crate::register::RegInput;
pub use crate::register_file::RegFile;
pub use rhdl_fpga::core::dff::DFF;
pub fn miette_report(err: RHDLError) -> String {
    let handler =
        miette::GraphicalReportHandler::new_themed(miette::GraphicalTheme::unicode_nocolor());
    let mut msg = String::new();
    handler.render_report(&mut msg, &err).unwrap();
    msg
}