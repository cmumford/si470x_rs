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

    info!("Pinging...");
    dev.ping().unwrap();
    info!("  ping success");

    let chip_info = dev.get_chip_info().unwrap();
    info!("{:?}", chip_info);

    loop {
        info!("Waiting...");
        let delay_start = Instant::now();
        while delay_start.elapsed() < Duration::from_millis(1_000) {}
    }
}
