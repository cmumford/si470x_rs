#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use esp_hal::{
    clock::CpuClock,
    delay::Delay,
    gpio::{Level, Output, OutputConfig},
    i2c::master::I2c,
    main,
    time::{Duration, Instant, Rate},
};
use log::{LevelFilter, info};
use si470x::{Si470x, driver_common::reset_radio_for_i2c};

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[main]
fn main() -> ! {
    esp_println::logger::init_logger(LevelFilter::Info);

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    cfg_if::cfg_if! {
        if #[cfg(feature = "esp32c6")] {
            let rst_gpio = peripherals.GPIO14;
            let mut sda_gpio = peripherals.GPIO19;
            let sen_gpio = peripherals.GPIO21;
            let scl_gpio = peripherals.GPIO20;
        } else {
            let rst_gpio = peripherals.GPIO1;
            let mut sda_gpio = peripherals.GPIO2;
            let sen_gpio = peripherals.GPIO3;
            let scl_gpio = peripherals.GPIO4;
        }
    }

    // Reset the Si4703 and put into I2C mode.
    let mut rst_pin = Output::new(rst_gpio, Level::High, OutputConfig::default());
    let mut sda_pin = Output::new(sda_gpio.reborrow(), Level::High, OutputConfig::default());
    let mut sen_pin = Output::new(sen_gpio, Level::High, OutputConfig::default());
    let mut delay = Delay::new();
    reset_radio_for_i2c(&mut rst_pin, &mut sda_pin, Some(&mut sen_pin), &mut delay).unwrap();

    info!("[main] Initializing I2C");
    let i2c = I2c::new(
        peripherals.I2C0,
        esp_hal::i2c::master::Config::default().with_frequency(Rate::from_khz(100)),
    )
    .unwrap()
    .with_scl(scl_gpio)
    .with_sda(sda_gpio);

    let mut dev = Si470x::new(i2c);
    dev.set_enable(true).unwrap();

    // Ping to confirm alive
    dev.ping().unwrap();
    info!("Ping OK");

    // Enable oscillator first
    dev.set_oscillator_enable(true).unwrap();
    info!("Oscillator enabled");
    delay.delay_millis(2000);

    // Enable radio
    dev.set_enable(true).unwrap();
    info!("Radio enabled");
    delay.delay_millis(1000);

    // Read full registers and log raw
    let registers = dev.read_all_registers().unwrap();
    info!("Raw 32-byte read: {:02X?}", registers);

    // Chip ID should be at buffer[14..16]
    let chip_id_raw = u16::from_be_bytes([registers[14], registers[15]]);
    info!("Raw ChipId u16: 0x{:04x}", chip_id_raw);

    // Then get_chip_info
    let chip_info = dev.get_chip_info().unwrap();
    info!("Chip info: {:?}", chip_info);

    let device_info = dev.get_device_info().unwrap();
    info!(
        "Device info: pn:0x{:x}, mfgid:0x{:x}",
        device_info.pn, device_info.mfgid
    );

    loop {
        info!("Waiting...");
        let delay_start = Instant::now();
        while delay_start.elapsed() < Duration::from_millis(1_000) {}
    }
}
