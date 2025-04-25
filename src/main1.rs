#![no_std]
#![no_main]

use defmt::info;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_rp::{
    adc::{Adc, Channel, Config, InterruptHandler},
    bind_interrupts,
    gpio::Pull,
};
use embassy_time::{Duration, Timer};
use panic_probe as _;

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
    let mut adc_channel = Channel::new_pin(peripherals.PIN_26, Pull::None); //new_pin(peripherals.PIN_26, None);
    let mut i = 0;
    let mut sum: u32 = 0;
    loop {
        // Read value from ADC
        let value = adc.read(&mut adc_channel).await.unwrap();
        info!("Value: {}", value);
        // Wait for 1 millisecond before sampling again
        Timer::after(Duration::from_millis(2)).await;
    }
}
