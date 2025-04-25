#![no_std]
#![no_main]

use defmt::info;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_rp::peripherals::USB;
use embassy_rp::{
    adc::{Adc, Channel, Config, InterruptHandler},
    bind_interrupts,
    gpio::Pull,
};
use embassy_time::{Duration, Timer};
use panic_probe as _;
// use embassy_rp::usb::{Driver, InterruptHandler};
// use embassy_usb::driver::{Endpoint, EndpointIn, EndpointOut};
// use embassy_usb::msos::{self, windows_version};
// use embassy_usb::{Builder, Config};
bind_interrupts!(struct Irqs {
    ADC_IRQ_FIFO => InterruptHandler;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let peripherals = embassy_rp::init(Default::default());
    info!("Device started");

    // Initialize ADC peripheral
    let config = Config::default();
    let mut adc = Adc::new(peripherals.ADC, Irqs, config);

    // Create an ADC channel for GPIO26
    let mut adc_channel = Channel::new_pin(peripherals.PIN_27, Pull::None); //new_pin(peripherals.PIN_26, None);
    let mut i = 0;
    let mut sum: u32 = 0;
    loop {
        // Read value from ADC
        let value = adc.read(&mut adc_channel).await.unwrap();
        info!("{}", value);
        // Wait for 1 millisecond before sampling again
        Timer::after(Duration::from_nanos(40)).await;
    }
}
