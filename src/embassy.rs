pub mod time_driver;

use time_driver::TimeDriver;

pub fn init(tc0: atmega_hal::pac::TC0) {
    TimeDriver::init(tc0)
}
