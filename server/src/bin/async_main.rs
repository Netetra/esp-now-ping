#![no_std]
#![no_main]

use defmt::info;
use embassy_executor::Spawner;
use esp_hal::{prelude::*, rng::Rng};
use esp_wifi::{esp_now::BROADCAST_ADDRESS, wifi::Protocol, EspWifiController};
use {defmt_rtt as _, esp_backtrace as _};

extern crate alloc;

macro_rules! mk_static {
    ($t:ty,$val:expr) => {{
        static STATIC_CELL: static_cell::StaticCell<$t> = static_cell::StaticCell::new();
        #[deny(unused_attributes)]
        let x = STATIC_CELL.uninit().write(($val));
        x
    }};
}

#[main]
async fn main(spawner: Spawner) {
    let peripherals = esp_hal::init({
        let mut config = esp_hal::Config::default();
        config.cpu_clock = CpuClock::max();
        config
    });

    esp_alloc::heap_allocator!(72 * 1024);

    let timg0 = esp_hal::timer::timg::TimerGroup::new(peripherals.TIMG0);
    let esp_wifi_ctrl = &*mk_static!(
        EspWifiController<'static>,
        esp_wifi::init(
            timg0.timer0,
            Rng::new(peripherals.RNG),
            peripherals.RADIO_CLK,
        )
        .unwrap()
    );

    let sys_timer = esp_hal::timer::systimer::SystemTimer::new(peripherals.SYSTIMER)
        .split::<esp_hal::timer::systimer::Target>();
    esp_hal_embassy::init(sys_timer.alarm0);
    info!("Embassy initialized!");

    let _ = spawner;

    let wifi = peripherals.WIFI;
    let mut esp_now = esp_wifi::esp_now::EspNow::new(&esp_wifi_ctrl, wifi).unwrap();
    esp_now.set_protocol(Protocol::P802D11LR.into()).unwrap();
    info!("esp-now version {}", esp_now.version().unwrap());

    loop {
        info!("Waiting...");
        let recv = esp_now.receive_async().await;
        let data = recv.data();
        esp_now.send_async(&BROADCAST_ADDRESS, data).await.unwrap();
        info!("Receive: {}", data);
    }
}
