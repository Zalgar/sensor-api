mod ltr390;

use actix_web::{web, App, HttpServer, HttpResponse, Responder, middleware::Logger}; // Import necessary Actix Web components
use linux_embedded_hal::I2cdev;  // Import I2C device from linux_embedded_hal
use serde::{Deserialize, Serialize}; // Import serialization/deserialization from Serde
use chrono::Utc; // Import Utc for timestamps
use std::fs; // Import filesystem operations
use std::fs::File; // Import file operations
use std::io::Write; // Import write operations
use env_logger::Env; // Import environment logger
use ltr390::LTR390; // Import LTR390 driver

// Configuration structure for the application
#[derive(Serialize, Deserialize)]
struct Config {
    network_port: u16, // Port for the web server
    i2c_address_decimal: u16, // I2C address of the LTR390 sensor
    i2c_bus_device_path: String, // Path to the I2C bus device
    bind_address: String, // Address to bind the web server to
    uv_index_factor: f32, // Factor to convert raw UVS data to UV index
}

// Default implementation for the Config struct
impl Default for Config {
    fn default() -> Self {
        Config {
            network_port: 5003, // Default network port
            i2c_address_decimal: 0x53, // Default I2C address (0x53 hex = 83 decimal)
            i2c_bus_device_path: String::from("/dev/i2c-1"), // Default I2C bus device path
            bind_address: String::from("0.0.0.0"), // Default bind address
            uv_index_factor: 0.000434, // Default UV index calculation factor (adjust as needed)
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
    uv_index: f32,      // UV index calculated value
    light_uv: u32,      // Raw UVS data from sensor
    light_ambient: f32, // Ambient light value
}

async fn get_sensor_data() -> impl Responder {
    let config = read_or_create_config();

    // Set up the I2C bus and LTR390 sensor
    let i2c_bus = match I2cdev::new(&config.i2c_bus_device_path) {
        Ok(bus) => bus,
        Err(e) => {
            eprintln!("Failed to open I2C bus: {:?}", e);
            return HttpResponse::InternalServerError().body("Failed to open I2C bus");
        }
    };

    // Create LTR390 sensor object with the correct I2C address
    let mut ltr390 = match LTR390::new(i2c_bus, config.i2c_address_decimal as u8) {
        Ok(sensor) => sensor,
        Err(e) => {
            eprintln!("Failed to create LTR390 sensor: {:?}", e);
            return HttpResponse::InternalServerError().body("Failed to create LTR390 sensor");
        }
    };

    // Initialize the LTR390 sensor
    if let Err(e) = ltr390.begin() {
        eprintln!("Failed to initialize LTR390 sensor: {:?}", e);
        return HttpResponse::InternalServerError().body("Failed to initialize LTR390 sensor");
    }

    // Read sensor data with proper delays between mode switches
    let uv_data = match ltr390.read_uvs() {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Failed to read UV sensor data: {:?}", e);
            return HttpResponse::InternalServerError().body("Failed to read UV sensor data");
        }
    };

    // Add delay between UVS and ALS readings to allow proper mode switching
    std::thread::sleep(std::time::Duration::from_millis(200));

    let als_data = match ltr390.read_als() {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Failed to read ALS sensor data: {:?}", e);
            return HttpResponse::InternalServerError().body("Failed to read ALS sensor data");
        }
    };

    // Calculate UV Index from raw UV data using configurable factor
    let uv_index = (uv_data as f32) * config.uv_index_factor;

    // Create sensor data response
    let sensor_data = SensorData {
        timestamp: Utc::now().to_rfc3339(),
        model: String::from("LTR390"),
        uv_index,
        light_uv: uv_data,
        light_ambient: als_data as f32,
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