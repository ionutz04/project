#![no_std]
#![no_main]

use defmt::{error, info};
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_rp::{
    adc::{Adc, Channel as adc_channel, Config as AdcConfig, InterruptHandler},
    bind_interrupts,
    gpio::Pull,
    peripherals::{DMA_CH0, UART1},
    uart::{Async, Config as UartConfig, UartTx},
};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};
use embassy_time::{Duration, Instant, Timer};
use panic_probe as _;

bind_interrupts!(struct Irqs {
    ADC_IRQ_FIFO => InterruptHandler;
});

const SAMPLING_RATE: u32 = 500_000;
const BUFFER_SIZE: usize = 2048;
const UART_BAUDRATE: u32 = 2_000_000;

static CHANNEL: Channel<CriticalSectionRawMutex, [u8; 8], 4> = Channel::new();

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    info!("System Initialized");

    // ADC Setup
    let mut adc = Adc::new(p.ADC, Irqs, AdcConfig::default());
    let mut adc_channel = adc_channel::new_pin(p.PIN_26, Pull::None);
    let mut uart_config = UartConfig::default();
    uart_config.baudrate = 2_000_000;
    // UART Setup
    let uart_tx = UartTx::new(p.UART1, p.PIN_4, p.DMA_CH0, uart_config);

    spawner.spawn(uart_handler(uart_tx)).unwrap();

    let mut measurement_state = MeasurementState::new();

    loop {
        let value = adc.read(&mut adc_channel).await.unwrap();
        measurement_state.process_sample(value);

        if measurement_state.buffer_full() {
            let (fwhm, amplitude, frequency) = measurement_state.calculate_metrics();
            send_measurements(fwhm, amplitude, frequency).await;
            measurement_state.reset();
        }

        Timer::after(Duration::from_micros(2)).await;
    }
}
struct MeasurementState {
    buffer: [u16; BUFFER_SIZE],
    index: usize,
    min: u16,
    max: u16,
    last_peak: Option<Instant>,
    frequency: f32,
    timestamp: f32,
}

impl MeasurementState {
    fn new() -> Self {
        Self {
            buffer: [0; BUFFER_SIZE],
            index: 0,
            min: u16::MAX,
            max: u16::MIN,
            last_peak: None,
            frequency: 0.0,
            timestamp: 0.0,
        }
    }

    fn process_sample(&mut self, value: u16) {
        self.buffer[self.index] = value;
        self.index = (self.index + 1) % BUFFER_SIZE;

        self.min = self.min.min(value);
        self.max = self.max.max(value);

        if is_peak(value, self.min, self.max) {
            let now = Instant::now();
            if let Some(last) = self.last_peak {
                let elapsed = now.duration_since(last).as_micros() as f32;
                if elapsed > 0.0 {
                    self.frequency = 1.0 / elapsed; // Hz
                }
            }
            self.last_peak = Some(now);
        }
    }

    fn buffer_full(&self) -> bool {
        self.index == 0
    }

    fn calculate_metrics(&self) -> (f32, f32, f32) {
        let fwhm = calculate_fwhm(&self.buffer);
        let amplitude = (self.max as f32) * 3.3 / 4095.0;
        (fwhm, amplitude, self.frequency)
    }

    fn reset(&mut self) {
        self.min = u16::MAX;
        self.max = u16::MIN;
        self.timestamp += BUFFER_SIZE as f32 / SAMPLING_RATE as f32;
    }
}

async fn send_measurements(fwhm: f32, amplitude: f32, frequency: f32) {
    let mut data = [0u8; 8];
    data[0..4].copy_from_slice(&frequency.to_le_bytes());
    data[4..8].copy_from_slice(&amplitude.to_le_bytes());

    CHANNEL.send(data).await;
}

#[embassy_executor::task]
async fn uart_handler(mut uart_tx: UartTx<'static, UART1, Async>) {
    info!("UART Handler Active");

    loop {
        let data = CHANNEL.receive().await;
        match uart_tx.write(&data).await {
            Ok(_) => info!(
                "Sent: Frequency={}Hz, Amp={}V",
                f32::from_le_bytes([data[0], data[1], data[2], data[3]]),
                f32::from_le_bytes([data[4], data[5], data[6], data[7]])
            ),
            Err(e) => error!("UART Error: {:?}", e),
        }
    }
}

fn calculate_fwhm(data: &[u16]) -> f32 {
    let max = data.iter().max().unwrap();
    let half_max = *max as f32 / 2.0;
    let mut left = 0;
    let mut right = data.len() - 1;

    while data[left] < half_max as u16 && left < right {
        left += 1;
    }
    while data[right] < half_max as u16 && right > left {
        right -= 1;
    }

    ((right - left) as f32 * 1_000_000.0) / SAMPLING_RATE as f32
}

fn is_peak(value: u16, min: u16, max: u16) -> bool {
    let threshold = min + (max - min) * 3 / 4;
    value > threshold
}

