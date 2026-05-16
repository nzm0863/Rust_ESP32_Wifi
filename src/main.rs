use anyhow::Result;

use embedded_svc::http::client::Client as HttpClient;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::hal::modem::Modem;
use esp_idf_svc::http::client::{Configuration, EspHttpConnection};
use esp_idf_svc::log::EspLogger;
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_svc::wifi::{
    AuthMethod, BlockingWifi, ClientConfiguration, Configuration as WifiConfiguration, EspWifi,
};

use embedded_svc::wifi::{ClientConfiguration as _, Configuration as _};

use std::io::Read;
use std::thread;
use std::time::Duration;

fn main() -> Result<()> {
    esp_idf_svc::sys::link_patches();
    EspLogger::initialize_default();

    log::info!("start");

    let peripherals = esp_idf_svc::hal::peripherals::Peripherals::take().unwrap();

    let sysloop = EspSystemEventLoop::take()?;
    let nvs = EspDefaultNvsPartition::take()?;

    let wifi = EspWifi::new(peripherals.modem, sysloop.clone(), Some(nvs))?;

    let mut wifi = BlockingWifi::wrap(wifi, sysloop)?;

    wifi.set_configuration(&WifiConfiguration::Client(ClientConfiguration {
        ssid: "rewrite-c".try_into().unwrap(),
        password: "welcome.rewrite".try_into().unwrap(),
        auth_method: AuthMethod::WPA2Personal,
        ..Default::default()
    }))?;

    log::info!("wifi start");

    wifi.start()?;

    log::info!("wifi connect");

    wifi.connect()?;

    log::info!("waiting netif");

    wifi.wait_netif_up()?;

    log::info!("wifi connected!");

    let connection = EspHttpConnection::new(&Configuration::default())?;

    let mut client = HttpClient::wrap(connection);

    log::info!("http client created");

    let mut request = client.get("http://192.168.1.92/e/index.php")?;

    let mut response = request.submit()?;

    log::info!("status: {}", response.status());

    let mut buf = [0_u8; 256];

    let size = response.read(&mut buf)?;

    let body = std::str::from_utf8(&buf[..size]).unwrap();

    log::info!("response: {}", body);

    loop {
        thread::sleep(Duration::from_secs(1));
    }
}
