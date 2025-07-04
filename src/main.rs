#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

use atmega_hal::{
    clock::MHz16,
    pac::USART0,
    port::{
        mode::{Input, Output},
        Pin, PD0, PD1,
    },
    usart::BaudrateExt,
};
use embassy_executor::Spawner;
use embassy_time::Timer;
use panic_halt as _;
use ufmt::uwriteln;

pub mod embassy;

type Usart = atmega_hal::usart::Usart<USART0, Pin<Input, PD0>, Pin<Output, PD1>, MHz16>;

#[embassy_executor::task]
async fn talk(mut serial: Usart) -> ! {
    loop {
        uwriteln!(serial, "Hello").unwrap();
        Timer::after_millis(1000).await;
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) -> ! {
    let dp = atmega_hal::Peripherals::take().unwrap();
    let pins = atmega_hal::pins!(dp);

    embassy::init(dp.TC0);

    let mut led = pins.pb5.into_output();
    led.set_low();

    let pin_tx = pins.pd1.into_output();
    let pin_rx = pins.pd0;
    let serial = atmega_hal::Usart::new(
        dp.USART0,
        pin_rx,
        pin_tx,
        BaudrateExt::into_baudrate::<MHz16>(57600),
    );

    spawner.spawn(talk(serial)).unwrap();

    unsafe { avr_device::interrupt::enable() };

    loop {
        Timer::after_millis(500).await;

        led.toggle();
    }
}
