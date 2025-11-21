use rhdl::prelude::*;
use miette;
mod doc;
mod alu;
mod register;
mod register_file;
mod ram;
// mod top;
mod control_unit;

use crate::{ram::sim_ram, register::{fixture::fixture_main, sim_reg}, register_file::sim_reg_file};
pub fn miette_report(err: RHDLError) -> String {
    let handler =
        miette::GraphicalReportHandler::new_themed(miette::GraphicalTheme::unicode());
    let mut msg = String::new();
    handler.render_report(&mut msg, &err).unwrap();
    msg
}

fn rhdl_main() -> Result<(), RHDLError> {
    fixture_main()?;
    sim_ram()?;
    sim_reg()?;
    sim_reg_file()?;
    Ok(())
}

fn main() {
    if let Err(e) = rhdl_main() {
        println!("{}", miette_report(e));
    }
}
