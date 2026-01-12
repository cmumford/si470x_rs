#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use embassy_executor::Spawner;
use embassy_time::Timer;
use esp_hal::{
    clock::CpuClock,
    delay::Delay,
    gpio::{AnyPin, DriveMode, Level, Output, OutputConfig},
    i2c::master::I2c,
    time::Rate,
    timer::timg::TimerGroup,
};
use log::{LevelFilter, info};
use si470x::Si470x;

extern crate alloc;

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

fn reset_radio<'d>(
    rst_gpio: impl Into<AnyPin<'d>>,
    sda_gpio: impl Into<AnyPin<'d>>,
    sen_gpio: impl Into<AnyPin<'d>>,
) {
    let mut rst_output = Output::new(
        rst_gpio.into(),
        Level::High,
        OutputConfig::default().with_drive_mode(DriveMode::PushPull),
    );
    let mut sda_output = Output::new(
        sda_gpio.into(),
        Level::High,
        OutputConfig::default().with_drive_mode(DriveMode::PushPull),
    );
    let mut sen_output = Output::new(
        sen_gpio.into(),
        Level::High,
        OutputConfig::default().with_drive_mode(DriveMode::PushPull),
    );
    rst_output.set_low();
    sda_output.set_low();
    sen_output.set_high(); // To select 2-wire mode.
    let delay = Delay::new();
    delay.delay(esp_hal::time::Duration::from_millis(5));
    rst_output.set_high();
    delay.delay(esp_hal::time::Duration::from_millis(5));
}

#[esp_rtos::main]
async fn main(_spawner: Spawner) -> ! {
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

    reset_radio(rst_gpio, sda_gpio.reborrow(), sen_gpio);

    esp_alloc::heap_allocator!(size: 32 * 1024);

    let timg1 = TimerGroup::new(peripherals.TIMG1);
    cfg_if::cfg_if! {
        if #[cfg(any(feature = "esp32",feature = "esp32s2",feature = "esp32s3"))] {
            esp_rtos::start(timg1.timer0);
        } else {
            use esp_hal::interrupt::software::SoftwareInterruptControl;
            let software_interrupt = SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);
            esp_rtos::start(timg1.timer0, software_interrupt.software_interrupt0);
        }
    }

    info!("[main] Initializing I2C");
    let i2c = I2c::new(
        peripherals.I2C0,
        esp_hal::i2c::master::Config::default().with_frequency(Rate::from_khz(100)),
    )
    .unwrap()
    .into_async()
    .with_scl(scl_gpio)
    .with_sda(sda_gpio);

    let mut dev = Si470x::new(i2c);

    info!("Pinging...");
    dev.ping_async().await.unwrap();
    info!("  ping success");
    loop {
        info!("Waiting...");
        Timer::after(embassy_time::Duration::from_millis(1_000)).await;
    }
}
