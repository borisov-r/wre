use crate::rotary::RotaryEncoderState;
use embedded_svc::wifi::{ClientConfiguration, Configuration, Wifi as _};
use esp_idf_hal::modem::Modem;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::http::server::{Configuration as HttpConfig, EspHttpServer};
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_svc::wifi::{BlockingWifi, EspWifi};
use log::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

// WiFi credentials must be set via environment variables at compile time
// Example: export WIFI_SSID='YourNetwork' && export WIFI_PASS='YourPassword'
const WIFI_SSID: &str = match option_env!("WIFI_SSID") {
    Some(v) => v,
    None => "WIFI_SSID_NOT_SET",
};

const WIFI_PASS: &str = match option_env!("WIFI_PASS") {
    Some(v) => v,
    None => "WIFI_PASS_NOT_SET",
};

#[derive(Serialize, Deserialize)]
struct SetAnglesRequest {
    angles: Vec<f32>,
}

#[derive(Serialize)]
struct StatusResponse {
    active: bool,
    angle: f32,
    target_angles: Vec<f32>,
    current_target_index: usize,
    output_on: bool,
}

pub fn start_webserver(
    encoder_state: RotaryEncoderState,
    modem: Modem,
) -> anyhow::Result<()> {
    info!("Initializing WiFi...");

    // Check if WiFi credentials are set
    if WIFI_SSID == "WIFI_SSID_NOT_SET" || WIFI_PASS == "WIFI_PASS_NOT_SET" {
        error!("WiFi credentials not set! Please set WIFI_SSID and WIFI_PASS environment variables.");
        error!("Example: export WIFI_SSID='YourNetwork' && export WIFI_PASS='YourPassword'");
        return Err(anyhow::anyhow!("WiFi credentials not configured"));
    }

    let sysloop = EspSystemEventLoop::take()?;
    let nvs = EspDefaultNvsPartition::take()?;

    let mut wifi = BlockingWifi::wrap(
        EspWifi::new(modem, sysloop.clone(), Some(nvs))?,
        sysloop,
    )?;

    wifi.set_configuration(&Configuration::Client(ClientConfiguration {
        ssid: WIFI_SSID.into(),
        password: WIFI_PASS.into(),
        ..Default::default()
    }))?;

    info!("Starting WiFi...");
    wifi.start()?;

    info!("Connecting to WiFi...");
    wifi.connect()?;

    info!("Waiting for IP address...");
    wifi.wait_netif_up()?;

    let ip_info = wifi.wifi().sta_netif().get_ip_info()?;
    info!("WiFi connected! IP: {}", ip_info.ip);

    // Start HTTP server
    let mut server = EspHttpServer::new(&HttpConfig::default())?;

    // Store encoder state for handlers
    let encoder_state_handlers = encoder_state.clone();

    // Serve HTML page
    server.fn_handler("/", embedded_svc::http::Method::Get, move |req| {
        let html = include_str!("../html/index.html");
        req.into_ok_response()?
            .write_all(html.as_bytes())?;
        Ok(())
    })?;

    // API: Get status
    let encoder_state_status = encoder_state_handlers.clone();
    server.fn_handler("/api/status", embedded_svc::http::Method::Get, move |req| {
        let status = StatusResponse {
            active: encoder_state_status.is_active(),
            angle: encoder_state_status.get_angle(),
            target_angles: encoder_state_status.get_target_angles(),
            current_target_index: encoder_state_status.get_current_target_index(),
            output_on: encoder_state_status.is_output_on(),
        };

        let json = serde_json::to_string(&status)
            .unwrap_or_else(|e| {
                error!("Failed to serialize status: {:?}", e);
                r#"{"error":"serialization_failed"}"#.to_string()
            });
        req.into_response(200, Some("OK"), &[("Content-Type", "application/json")])?
            .write_all(json.as_bytes())?;
        Ok(())
    })?;

    // API: Set angles
    let encoder_state_set = encoder_state_handlers.clone();
    server.fn_handler("/api/set", embedded_svc::http::Method::Post, move |mut req| {
        let mut buf = vec![0u8; 1024]; // Increased from 512 to support more angles
        let len = req.read(&mut buf)?;
        
        match serde_json::from_slice::<SetAnglesRequest>(&buf[..len]) {
            Ok(request) => {
                info!("Setting target angles: {:?}", request.angles);
                encoder_state_set.set_target_angles(request.angles);
                
                req.into_response(200, Some("OK"), &[("Content-Type", "application/json")])?
                    .write_all(b"{\"status\":\"ok\"}")?;
            }
            Err(e) => {
                error!("Failed to parse request: {:?}", e);
                let error_msg = format!(r#"{{"status":"error","message":"Invalid JSON: {}"}}"#, e);
                req.into_response(400, Some("Bad Request"), &[("Content-Type", "application/json")])?
                    .write_all(error_msg.as_bytes())?;
            }
        }
        Ok(())
    })?;

    // API: Stop encoder
    let encoder_state_stop = encoder_state_handlers.clone();
    server.fn_handler("/api/stop", embedded_svc::http::Method::Post, move |req| {
        info!("Stopping encoder");
        encoder_state_stop.stop();
        
        req.into_response(200, Some("OK"), &[("Content-Type", "application/json")])?
            .write_all(b"{\"status\":\"ok\"}")?;
        Ok(())
    })?;

    info!("Web server started at http://{}", ip_info.ip);
    info!("Open this URL in your browser to control the encoder");

    // Keep the server running
    loop {
        thread::sleep(Duration::from_secs(1));
    }
}
