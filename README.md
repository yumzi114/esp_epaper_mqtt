# esp_epaper_mqtt
## Create cfg.toml
```rust
[mqtt]
mqtt_user = ""
mqtt_pass = "zz"
mqtt_host = ""
wifi_ssid = ""
wifi_psk = ""
```
## Edit build.rs 
look like this
```rust
#[toml_cfg::toml_config]
pub struct Config {
    #[default("localhost")]
    mqtt_host: &'static str,
    #[default("")]
    mqtt_user: &'static str,
    #[default("")]
    mqtt_pass: &'static str,
    #[default("")]
    wifi_ssid: &'static str,
    #[default("")]
    wifi_psk: &'static str,
}

fn main() {
    if !std::path::Path::new("cfg.toml").exists() {
        panic!("You need to create a `cfg.toml` file with your Wi-Fi credentials! Use `cfg.toml.example` as a template.");
    }
    let app_config = CONFIG;
    if app_config.wifi_ssid == "your wifi" || app_config.wifi_psk == "your wifi pw" {
        panic!("You need to set the Wi-Fi credentials in `cfg.toml`!");
    }
    if app_config.mqtt_host == "mqtt url"
        || app_config.mqtt_user == "mqtt user"
        || app_config.mqtt_pass == "mqtt pw"
    {
        panic!("You need to set the MQTT credentials in `cfg.toml`!");
    }
    embuild::espidf::sysenv::output();
}
```
and appand Cargo.toml
```rust
[dependencies]
toml-cfg      = "=0.1.3"
[build-dependencies]
toml-cfg = "=0.1.3"

```
you can use cfg config setting
but this report is not used

mqtt tls auth is slow
wait for wifi, mqtt connection