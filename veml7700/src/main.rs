mod veml7700;

use actix_web::{web, App, HttpServer, HttpResponse, Responder, middleware::Logger}; // Import necessary Actix Web components
use linux_embedded_hal::I2cdev;  // Import I2C device from linux_embedded_hal
use serde::{Deserialize, Serialize}; // Import serialization/deserialization from Serde
use chrono::Utc; // Import Utc for timestamps
use std::fs; // Import filesystem operations
use std::fs::File; // Import file operations
use std::io::Write; // Import write operations
use env_logger::Env; // Import environment logger
use veml7700::VEML7700; // Import VEML7700 driver

// Configuration structure for the application
#[derive(Serialize, Deserialize)]
struct Config {
    network_port: u16, // Port for the web server
    i2c_address_decimal: u16, // I2C address of the VEML7700 sensor
    i2c_bus_device_path: String, // Path to the I2C bus device
    bind_address: String, // Address to bind the web server to
    als_gain: u16, // ALS gain setting (0=1x, 1=2x, 2=1/8x, 3=1/4x)
    integration_time: u16, // Integration time setting (0=25ms, 1=50ms, 2=100ms, 3=200ms, 4=400ms, 5=800ms)
}

// Default implementation for the Config struct
impl Default for Config {
    fn default() -> Self {
        Config {
            network_port: 5005, // Default network port (different from others)
            i2c_address_decimal: 0x10, // Default I2C address (0x10 hex = 16 decimal)
            i2c_bus_device_path: String::from("/dev/i2c-1"), // Default I2C bus device path
            bind_address: String::from("0.0.0.0"), // Default bind address
            als_gain: 0, // Default gain: 0 = 1x
            integration_time: 2, // Default integration time: 2 = 100ms
        }
    }
}

// Function to read the configuration from a file or create a default one if it doesn't exist
fn read_or_create_config() -> Config {
    let config_path = "config.json";

    // Check if the configuration file exists
    if std::path::Path::new(config_path).exists() {
        // If it exists, read and parse the configuration
        let config_data = fs::read_to_string(config_path).unwrap();
        match serde_json::from_str(&config_data) {
            Ok(config) => return config,
            Err(e) => {
                eprintln!("Error parsing config file: {}. Using default configuration.", e);
            }
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
    light_ambient: f32, // Ambient light value in lux
    light_white: u16,   // White light channel raw value
}

async fn get_sensor_data() -> impl Responder {
    let config = read_or_create_config();

    // Set up the I2C bus and VEML7700 sensor
    let i2c_bus = match I2cdev::new(&config.i2c_bus_device_path) {
        Ok(bus) => bus,
        Err(e) => {
            eprintln!("Failed to open I2C bus: {:?}", e);
            return HttpResponse::InternalServerError().body("Failed to open I2C bus");
        }
    };

    // Create VEML7700 sensor object with the correct I2C address
    let mut veml7700 = match VEML7700::new(i2c_bus, config.i2c_address_decimal as u8) {
        Ok(sensor) => sensor,
        Err(e) => {
            eprintln!("Failed to create VEML7700 sensor: {:?}", e);
            return HttpResponse::InternalServerError().body("Failed to create VEML7700 sensor");
        }
    };

    // Initialize the VEML7700 sensor
    if let Err(e) = veml7700.begin() {
        eprintln!("Failed to initialize VEML7700 sensor: {:?}", e);
        return HttpResponse::InternalServerError().body("Failed to initialize VEML7700 sensor");
    }

    // Configure sensor with settings from config file
    if let Err(e) = veml7700.set_gain(config.als_gain) {
        eprintln!("Failed to set sensor gain: {:?}", e);
        return HttpResponse::InternalServerError().body("Failed to set sensor gain");
    }

    if let Err(e) = veml7700.set_integration_time(config.integration_time) {
        eprintln!("Failed to set integration time: {:?}", e);
        return HttpResponse::InternalServerError().body("Failed to set integration time");
    }

    // Read sensor data
    let lux_data = match veml7700.read_lux() {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Failed to read lux data: {:?}", e);
            return HttpResponse::InternalServerError().body("Failed to read lux data");
        }
    };

    let white_data = match veml7700.read_white() {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Failed to read white light data: {:?}", e);
            return HttpResponse::InternalServerError().body("Failed to read white light data");
        }
    };

    // Create sensor data response
    let sensor_data = SensorData {
        timestamp: Utc::now().to_rfc3339(),
        model: String::from("VEML7700"),
        light_ambient: lux_data,
        light_white: white_data,
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
