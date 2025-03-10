#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::Timer;

use esp_backtrace as _;
use esp_hal::clock::CpuClock;
use esp_hal::i2c;
use esp_hal::peripherals::Peripherals;
use esp_hal::time::Duration;
use log::info;

use espnow_mesh_temp_monitoring_rs::gateway_lib::display::{
    configure_text_style, display_message, DisplayData, MqttLevelUnit, WifiLevelUnit,
};
use espnow_mesh_temp_monitoring_rs::gateway_lib::greet::log_init_complete;

use ssd1306::{prelude::*, size::DisplaySize128x64, I2CDisplayInterface, Ssd1306Async};

extern crate alloc;

const HEAP_SIZE_KB: usize = 128;
const OLED_ADDRESS: u8 = 0x3C;

async fn allocate_heap() {
    esp_alloc::heap_allocator!(HEAP_SIZE_KB); // NOTE: Do I need a heap for wifi ? or else ?
}

#[esp_hal_embassy::main]
async fn main(_spawner: Spawner) {
    // ********** Hardware init and wifi ********** //
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals: Peripherals = esp_hal::init(config);

    esp_println::logger::init_logger_from_env();

    allocate_heap().await;

    // Embassy init
    let timer0 = esp_hal::timer::timg::TimerGroup::new(peripherals.TIMG1);
    esp_hal_embassy::init(timer0.timer0);
    info!("Embassy initialized!");

    // I2C init for display driver
    //let mut gpio_16 = gpio::Output::new(peripherals.GPIO16, gpio::Level::Low);
    //let mut gpio_17 = gpio::Output::new(peripherals.GPIO17, gpio::Level::Low);
    let i2c_module = i2c::master::I2c::new(peripherals.I2C0, i2c::master::Config::default())
        .unwrap()
        .with_sda(peripherals.GPIO16)
        .with_scl(peripherals.GPIO17)
        .into_async();

    // initialize the display
    let interface = I2CDisplayInterface::new_custom_address(i2c_module, OLED_ADDRESS);
    let mut display = Ssd1306Async::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();
    display.init().await.unwrap();
    let text_style = configure_text_style();

    // Initialize the data to display
    let wifi_status_display = WifiLevelUnit {
        msg: "Wifi",
        level: 0,
        unit: "%",
    };
    let mqtt_status_display = MqttLevelUnit::new("MQTT client", 0);
    let time_status_display = Duration::micros_at_least(0);
    let mut device_data = DisplayData {
        wifi: wifi_status_display,
        mqtt_client: mqtt_status_display,
        us_since_last_update: time_status_display,
    };

    display_message(&mut display, &text_style, &device_data).await;

    // WIFI setup
    //let timer1 = esp_hal::timer::timg::TimerGroup::new(peripherals.TIMG0);
    //let _init = esp_wifi::init(
    //    timer1.timer0,
    //    esp_hal::rng::Rng::new(peripherals.RNG),
    //    peripherals.RADIO_CLK,
    //)
    //.unwrap();
    //info!("Wifi configured!");

    log_init_complete().await;

    // ********** init end ********** //

    loop {
        // test refresh
        info!("Mod wifi and mqtt");
        display.clear_buffer();
        device_data.wifi.level = 25;
        device_data.mqtt_client.update_status(1);
        display_message(&mut display, &text_style, &device_data).await;
        Timer::after_secs(2).await;

        info!("clearing and writing 100");
        display.clear_buffer();
        device_data.wifi.level = 100;
        device_data.mqtt_client.update_status(2);
        display_message(&mut display, &text_style, &device_data).await;
        Timer::after_secs(2).await;
    }

    // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/v0.23.1/examples/src/bin
}
