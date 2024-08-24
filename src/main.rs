use anyhow::{self, bail, Error, Ok};
use log::info;
use epd_waveshare::{color,epd2in9_v2::*, graphics::DisplayRotation, prelude::*};
use esp_idf_hal::delay::Ets;
use esp_idf_hal::gpio::*;
use esp_idf_hal::spi::{config::Config, SpiDeviceDriver};
use esp_idf_hal::{peripherals::Peripherals, spi::SpiDriverConfig};
use embedded_graphics::{
    mono_font::MonoTextStyleBuilder,
    pixelcolor::BinaryColor::On as Black,
    pixelcolor::BinaryColor::{self, Off as White},
    prelude::*,
    primitives::{Circle, Line, PrimitiveStyleBuilder},
    text::{Baseline, Text, TextStyleBuilder},
};

use embedded_svc::wifi::{AuthMethod, ClientConfiguration, Configuration};

use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_svc::wifi::EspWifi;
use esp_idf_svc::mqtt::client::{EspMqttClient, EventPayload, MqttClientConfiguration, MqttProtocolVersion, QoS};
use heapless::String;
use core::str;
use std::thread;
use std::{
    thread::sleep,
    time::Duration,
};
use esp_idf_hal::gpio::PinDriver;
use std::{convert::TryFrom};
use portable_atomic::AtomicU32;
fn main()->anyhow::Result<()> {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();
    let peripherals = Peripherals::take().unwrap();
    let sysloop = EspSystemEventLoop::take()?;
    let nvs = EspDefaultNvsPartition::take()?;
    let mut wifi = EspWifi::new(peripherals.modem, sysloop, Some(nvs))?;
    //WIFI CONFIG
    let ssid: String<32> = String::try_from("").unwrap();
    let password: String<64> = String::try_from("").unwrap();
 
    let spi = peripherals.spi2;
    let sclk = peripherals.pins.gpio6;
    let miso = peripherals.pins.gpio21;
    let mosi = PinDriver::input(peripherals.pins.gpio7)?;
    let cs = PinDriver::output(peripherals.pins.gpio3)?;
    let dc = PinDriver::output(peripherals.pins.gpio8)?;
    let rst = PinDriver::output(peripherals.pins.gpio5)?;
    let mut delay = Ets;
    let config = Config::new().baudrate(112500.into());
    let mut device = SpiDeviceDriver::new_single(
        spi,
        sclk,
        miso,
        Option::<Gpio2>::None,
        Option::<AnyIOPin>::None,
        &SpiDriverConfig::default(),
        &config,
    )?;
    let mut epd = Epd2in9::new(
        &mut device, 
        cs, 
        mosi, 
        dc, 
        rst, 
        &mut delay)?;
    println!("Init done!");
    let mut display = Display2in9::default();
    display.set_rotation(DisplayRotation::Rotate270);
    epd.update_frame(&mut device, display.buffer(), &mut delay)?;
    epd.display_frame(&mut device, &mut delay)?;


    wifi.set_configuration(&Configuration::Client(ClientConfiguration {
        ssid,
        password,
        //your wifi AuthMethod
        auth_method: AuthMethod::WPA3Personal,
        ..Default::default()
    }))?;

    esp_idf_svc::log::EspLogger::initialize_default();
    wifi.start()?;
    wifi.connect()?;
    
    let text_style = TextStyleBuilder::new().baseline(Baseline::Top).build();

    let c_style = MonoTextStyleBuilder::new()
            .font(&embedded_graphics::mono_font::ascii::FONT_10X20)
            .text_color(Black)
            .background_color(White)
            .build();
    println!("Wifi Connected");
    let mqtt_config = MqttClientConfiguration{
        protocol_version:Some(MqttProtocolVersion::V3_1_1),
        // mqtt auth user, pw and connect client id
        client_id:Some("ESP"),
        username:Some(""),
        password:Some(""),
        ..Default::default()
    };
    // MQTT run closure (can run main loop)
    // let mut client = EspMqttClient::new_cb(
    //     "mqtt://192.168.0.106:1883/iot", 
    //     &mqtt_config,
    //     move |message_event| {
    //         match message_event.payload() {
    //             EventPayload::Received{ id, topic, data, details }=>{
                    
    //                 // if wifi.is_connected().unwrap(){
    //                 //     println!("wifi {:?}",std::str::from_utf8(data));    
    //                 // }
    //                 // let _ = Text::with_text_style("HELLOW WORLD", Point::new(0, 0), c_style, text_style)
    //                 // .draw(&mut display);
    //                 // epd.update_frame(&mut device, display.buffer(), &mut delay).unwrap();
    //                 // epd.display_frame(&mut device, &mut delay).unwrap();
    //                 TEST_AUTO.store(TEST_AUTO.load(Ordering::Relaxed)+1, Ordering::Relaxed);
    //                 println!("DATA {:?}",std::str::from_utf8(data));
    //             }

    //             _=>println!("ERROR"),
    //         }
    //     }
    // )?;
    let (mut client, mut con)=EspMqttClient::new("mqtt://192.168.0.106:1883/iot", &mqtt_config)?;
    // MQTT run thread(thread running)
    // thread::spawn(move || {
    //     info!("MQTT Listening for messages");

    //     while let core::result::Result::Ok(msg) = con.next() {
    //         match msg.payload() {
    //             EventPayload::Received{
    //                 id,
    //                 topic,
    //                 data,
    //                 details,
    //             }=>{
                    
    //                 println!("{:?}",str::from_utf8(data).unwrap());
    //             }
    //             _=>println!("No")
    //         }
    //     }

    //     info!("MQTT connection loop exit");
    // });
    while let core::result::Result::Ok(msg) = con.next() {
        match msg.payload() {
            EventPayload::Received{
                id,
                topic,
                data,
                details,
            }=>{
                display.clear_buffer(Color::White);
                display.clear(BinaryColor::Off).unwrap();
                let _ = Text::with_text_style(str::from_utf8(data).unwrap(), Point::new(0, 0), c_style, text_style)
                .draw(&mut display);
                epd.update_frame(&mut device, display.buffer(), &mut delay)?;
                epd.display_frame(&mut device, &mut delay)?;
            }
            _=>{
    
            }
        }
    }

    Ok(())
}



