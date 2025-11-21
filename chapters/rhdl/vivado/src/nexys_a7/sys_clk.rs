use rhdl::prelude::*;
use rhdl_bsp::{bga_pin, constraints::IOStandard, drivers::{get_clock_input,xilinx::open_collector::{self, Options}}};

pub fn sys_clock<T: CircuitIO>(path: &Path) -> Result<Driver<T>,RHDLError> {
    // Throw error if not clock input
    let _ = get_clock_input::<T>(path)?;
    let mut driver = open_collector::build::<T>(
        "sysclk",
        path,
        &Options {
            io_standard: IOStandard::LowVoltageCMOS_3v3,
            pins: vec![
                bga_pin!(E, 5)
            ]
        }
    )?;
    driver.constraints += "create_clock -period 10.00 -waveform {0 5} [get_ports sysclk]\n";
    Ok(driver)
}