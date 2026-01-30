use crate::rotary::RotaryEncoderState;
use embedded_svc::io::Write;
use embedded_svc::wifi::{AccessPointConfiguration, AuthMethod, ClientConfiguration, Configuration};
use esp_idf_hal::modem::Modem;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::http::server::{Configuration as HttpConfig, EspHttpServer};
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_svc::wifi::{BlockingWifi, EspWifi};
use log::*;
use serde::{Deserialize, Serialize};
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

// Default AP (Access Point) configuration for fallback mode
const AP_SSID: &str = "abkant";
const AP_PASS: &str = "123456789";

fn setup_ap_mode(wifi: &mut BlockingWifi<EspWifi<'static>>) -> anyhow::Result<std::net::Ipv4Addr> {
    info!("Configuring Access Point mode...");
    info!("AP SSID: {}", AP_SSID);
    info!("AP Password: {}", AP_PASS);
    
    wifi.set_configuration(&Configuration::AccessPoint(AccessPointConfiguration {
        ssid: AP_SSID.try_into().unwrap(),
        password: AP_PASS.try_into().unwrap(),
        auth_method: AuthMethod::WPA2Personal,
        ..Default::default()
    }))?;

    info!("Starting Access Point...");
    wifi.start()?;
    
    info!("Waiting for Access Point to be ready...");
    wifi.wait_netif_up()?;
    
    let ip_info = wifi.wifi().ap_netif().get_ip_info()?;
    info!("Access Point started! IP: {}", ip_info.ip);
    info!("Connect to WiFi network '{}' with password '{}'", AP_SSID, AP_PASS);
    
    Ok(ip_info.ip)
}

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

    let sysloop = EspSystemEventLoop::take()?;
    let nvs = EspDefaultNvsPartition::take()?;

    let mut wifi = BlockingWifi::wrap(
        EspWifi::new(modem, sysloop.clone(), Some(nvs))?,
        sysloop,
    )?;

    let ip_address;

    // Try to connect to configured WiFi network (if credentials are set)
    if WIFI_SSID != "WIFI_SSID_NOT_SET" && WIFI_PASS != "WIFI_PASS_NOT_SET" {
        info!("Attempting to connect to WiFi network: {}", WIFI_SSID);
        
        wifi.set_configuration(&Configuration::Client(ClientConfiguration {
            ssid: WIFI_SSID.try_into().unwrap(),
            password: WIFI_PASS.try_into().unwrap(),
            ..Default::default()
        }))?;

        wifi.start()?;
        
        // Try to connect with a timeout
        match wifi.connect() {
            Ok(_) => {
                info!("Connected to WiFi network");
                match wifi.wait_netif_up() {
                    Ok(_) => {
                        let ip_info = wifi.wifi().sta_netif().get_ip_info()?;
                        info!("WiFi connected! IP: {}", ip_info.ip);
                        ip_address = ip_info.ip;
                    }
                    Err(e) => {
                        error!("Failed to get IP address: {:?}", e);
                        info!("Falling back to Access Point mode...");
                        wifi.stop()?;
                        ip_address = setup_ap_mode(&mut wifi)?;
                    }
                }
            }
            Err(e) => {
                error!("Failed to connect to WiFi network: {:?}", e);
                info!("Falling back to Access Point mode...");
                wifi.stop()?;
                ip_address = setup_ap_mode(&mut wifi)?;
            }
        }
    } else {
        // No WiFi credentials configured, start in AP mode
        info!("No WiFi credentials configured, starting in Access Point mode...");
        ip_address = setup_ap_mode(&mut wifi)?;
    }

    // Start HTTP server
    let mut server = EspHttpServer::new(&HttpConfig::default())?;

    // Store encoder state for handlers
    let encoder_state_handlers = encoder_state.clone();

    // Serve HTML page
    server.fn_handler("/", embedded_svc::http::Method::Get, move |req| {
        let html = include_str!("../html/index.html");
        req.into_ok_response()?
            .write_all(html.as_bytes())?;
        Ok::<(), anyhow::Error>(())
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
        Ok::<(), anyhow::Error>(())
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
        Ok::<(), anyhow::Error>(())
    })?;

    // API: Stop encoder
    let encoder_state_stop = encoder_state_handlers.clone();
    server.fn_handler("/api/stop", embedded_svc::http::Method::Post, move |req| {
        info!("Stopping encoder");
        encoder_state_stop.stop();
        
        req.into_response(200, Some("OK"), &[("Content-Type", "application/json")])?
            .write_all(b"{\"status\":\"ok\"}")?;
        Ok::<(), anyhow::Error>(())
    })?;

    info!("Web server started at http://{}", ip_address);
    info!("Open this URL in your browser to control the encoder");

    // Keep the server running
    loop {
        thread::sleep(Duration::from_secs(1));
    }
}
