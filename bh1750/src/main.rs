mod bh1750;

use actix_web::{web, App, HttpServer, HttpResponse, Responder, middleware::Logger}; // Import necessary Actix Web components
use linux_embedded_hal::I2cdev;  // Import I2C device from linux_embedded_hal
use serde::{Deserialize, Serialize}; // Import serialization/deserialization from Serde
use chrono::Utc; // Import Utc for timestamps
use std::fs; // Import filesystem operations
use std::fs::File; // Import file operations
use std::io::Write; // Import write operations
use env_logger::Env; // Import environment logger
use bh1750::BH1750; // Import BH1750 driver

// Configuration structure for the application
#[derive(Serialize, Deserialize)]
struct Config {
    network_port: u16, // Port for the web server
    i2c_address_decimal: u16, // I2C address of the BH1750 sensor
    i2c_bus_device_path: String, // Path to the I2C bus device
    bind_address: String, // Address to bind the web server to
}

// Default implementation for the Config struct
impl Default for Config {
    fn default() -> Self {
        Config {
            network_port: 5004, // Default network port (different from LTR390)
            i2c_address_decimal: 0x23, // Default I2C address (0x23 hex = 35 decimal)
            i2c_bus_device_path: String::from("/dev/i2c-1"), // Default I2C bus device path
            bind_address: String::from("0.0.0.0"), // Default bind address
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
}

async fn get_sensor_data() -> impl Responder {
    let config = read_or_create_config();

    // Set up the I2C bus and BH1750 sensor
    let i2c_bus = match I2cdev::new(&config.i2c_bus_device_path) {
        Ok(bus) => bus,
        Err(e) => {
            eprintln!("Failed to open I2C bus: {:?}", e);
            return HttpResponse::InternalServerError().body("Failed to open I2C bus");
        }
    };

    // Create BH1750 sensor object with the correct I2C address
    let mut bh1750 = match BH1750::new(i2c_bus, config.i2c_address_decimal as u8) {
        Ok(sensor) => sensor,
        Err(e) => {
            eprintln!("Failed to create BH1750 sensor: {:?}", e);
            return HttpResponse::InternalServerError().body("Failed to create BH1750 sensor");
        }
    };

    // Initialize the BH1750 sensor
    if let Err(e) = bh1750.begin() {
        eprintln!("Failed to initialize BH1750 sensor: {:?}", e);
        return HttpResponse::InternalServerError().body("Failed to initialize BH1750 sensor");
    }

    // Read sensor data
    let light_data = match bh1750.read_light_level() {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Failed to read light sensor data: {:?}", e);
            return HttpResponse::InternalServerError().body("Failed to read light sensor data");
        }
    };

    // Create sensor data response
    let sensor_data = SensorData {
        timestamp: Utc::now().to_rfc3339(),
        model: String::from("BH1750"),
        light_ambient: light_data,
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
