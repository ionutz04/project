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
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_time::{Duration, Instant, Timer};
use panic_probe as _;

bind_interrupts!(struct Irqs {
    ADC_IRQ_FIFO => InterruptHandler;
});

const SAMPLING_RATE: u32 = 500_000; // 500 KS/s
const BUFFER_SIZE: usize = 4096; // Mărește pentru a captura ≥1 ciclu

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let peripherals = embassy_rp::init(Default::default());
    info!("Device started");

    let config = Config::default();
    let mut adc = Adc::new(peripherals.ADC, Irqs, config);
    let mut adc_channel = Channel::new_pin(peripherals.PIN_27, Pull::None);

    let mut last_peak_time: Option<Instant> = None;
    let mut frequency = 0.0;

    let mut min = u16::MAX;
    let mut max = u16::MIN;
    let mut buffer: [u16; BUFFER_SIZE] = [0; BUFFER_SIZE];
    let mut index = 0;
    let mut timestamp = 0.0;

    loop {
        let value = adc.read(&mut adc_channel).await.unwrap();
        buffer[index] = value;
        index = (index + 1) % BUFFER_SIZE;

        // Update min/max
        if value < min {
            min = value;
        }
        if value > max {
            max = value;
        }

        // Detectare vârfuri cu prag adaptiv
        if is_peak(value, min, max) {
            let current_time = Instant::now();
            if let Some(last_time) = last_peak_time {
                let elapsed = current_time.duration_since(last_time).as_micros() as f32;
                if elapsed > 0.0 {
                    frequency = 1.0 / elapsed; // Hz
                }
            }
            last_peak_time = Some(current_time);
        }

        // FWHM și afișare
        if index == 0 {
            let fwhm = calculate_fwhm(&buffer);
            let amplitude = (max as f32 - min as f32) * 3.3 / 4095.0;
            info!(
                "ADC: {} | FWHM: {} μs | Freq: {} Hz | Amp: {} V | Time: {} s",
                value,
                fwhm,
                frequency * 1000.0,
                amplitude,
                timestamp
            );
            timestamp += 1.0 / SAMPLING_RATE as f32; // Actualizează timpul
        }

        Timer::after(Duration::from_micros(2)).await;
    }
}

fn calculate_fwhm(data: &[u16]) -> f32 {
    let max = data.iter().copied().max().unwrap();
    let half_max = max as f32 / 2.0;

    let mut left = 0;
    let mut right = data.len() - 1;

    while data[left] < half_max as u16 {
        left += 1;
    }

    while data[right] < half_max as u16 {
        right -= 1;
    }

    let width = (right as f32 - left as f32) / SAMPLING_RATE as f32;
    width * 1_000_000.0
}

fn is_peak(value: u16, min: u16, max: u16) -> bool {
    // Prag dinamic: 75% din amplitudine
    let threshold = min + (max - min) * 3 / 4;
    value > threshold
}
