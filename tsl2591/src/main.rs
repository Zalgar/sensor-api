use actix_web::{web, App, HttpServer, HttpResponse, Responder, middleware::Logger};
use linux_embedded_hal::I2cdev;
use serde::{Deserialize, Serialize};
use chrono::Utc;
use std::fs;
use std::fs::File;
use std::io::Write;
use env_logger::Env;
use log::error;

mod tsl2591;
use tsl2591::TSL2591;

// Configuration structure for the application
#[derive(Serialize, Deserialize)]
struct Config {
    network_port: u16,
    i2c_address_decimal: u16,
    i2c_bus_device_path: String,
    bind_address: String,
    gain: u16,
    integration_time: u16,
}

// Default implementation for the Config struct
impl Default for Config {
    fn default() -> Self {
        Config {
            network_port: 5006,
            i2c_address_decimal: 0x29,
            i2c_bus_device_path: String::from("/dev/i2c-1"),
            bind_address: String::from("0.0.0.0"),
            gain: 1, // Default gain (MEDIUM)
            integration_time: 1, // Default integration time (100ms)
        }
    }
}

// Function to read the configuration from a file or create a default one if it doesn't exist
fn read_or_create_config() -> Config {
    let config_path = "config.json";
    if let Ok(config_data) = fs::read_to_string(config_path) {
        if let Ok(config) = serde_json::from_str(&config_data) {
            return config;
        }
    }

    let default_config = Config::default();
    let config_data = serde_json::to_string_pretty(&default_config).unwrap();
    let mut file = File::create(config_path).unwrap();
    file.write_all(config_data.as_bytes()).unwrap();
    default_config
}

// Structure to hold sensor data
#[derive(Serialize)]
struct SensorData {
    timestamp: String,
    model: String,
    visible: u16,
    ir: u16,
    lux: f32,
}

async fn get_sensor_data() -> impl Responder {
    let config = read_or_create_config();

    let i2c_bus = match I2cdev::new(&config.i2c_bus_device_path) {
        Ok(bus) => bus,
        Err(e) => {
            error!("Failed to open I2C bus: {}", e);
            return HttpResponse::InternalServerError().body("Failed to open I2C bus");
        }
    };

    let mut sensor = match TSL2591::new(i2c_bus, config.i2c_address_decimal as u8) {
        Ok(sensor) => sensor,
        Err(e) => {
            error!("Failed to create TSL2591 sensor: {}", e);
            return HttpResponse::InternalServerError().body("Failed to create sensor");
        }
    };

    if let Err(e) = sensor.init(config.gain, config.integration_time) {
        error!("Failed to initialize TSL2591 sensor: {}", e);
        return HttpResponse::InternalServerError().body("Failed to initialize sensor");
    }

    let (visible, ir, lux) = match sensor.read_lux() {
        Ok(data) => data,
        Err(e) => {
            error!("Failed to read TSL2591 sensor data: {}", e);
            return HttpResponse::InternalServerError().body("Failed to read sensor data");
        }
    };

    let sensor_data = SensorData {
        timestamp: Utc::now().to_rfc3339(),
        model: String::from("TSL2591"),
        visible,
        ir,
        lux,
    };

    HttpResponse::Ok().json(sensor_data)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(Env::default().default_filter_or("info"));

    let config = read_or_create_config();

    HttpServer::new(|| {
        App::new()
            .wrap(Logger::default())
            .route("/sensor_data", web::get().to(get_sensor_data))
    })
    .bind((config.bind_address.as_str(), config.network_port))? // Use bind_address from config
    .run()
    .await
}