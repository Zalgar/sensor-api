use actix_web::{web, App, HttpServer, HttpResponse, Responder, middleware::Logger}; // Import necessary Actix Web components
use bme280::i2c::BME280; // Import BME280 sensor library
use linux_embedded_hal::{I2cdev, Delay};  // Import I2C device and delay from linux_embedded_hal
use serde::{Deserialize, Serialize}; // Import serialization/deserialization from Serde
//use chrono::Utc; // Import Utc for timestamps
use std::fs; // Import filesystem operations
use std::fs::File; // Import file operations
use std::io::Write; // Import write operations
use env_logger::Env; // Import environment logger

// Configuration structure for the application
#[derive(Serialize, Deserialize)]
struct Config {
    network_port: u16, // Port for the web server
    sea_level_pressure: f32, // Sea level pressure for altitude calculations
    i2c_address_decimal: u16, // I2C address of the BME280 sensor
    i2c_bus_device_path: String, // Path to the I2C bus device
    bind_address: String, // Address to bind the web server to
}

// Default implementation for the Config struct
impl Default for Config {
    fn default() -> Self {
        Config {
            network_port: 5000, // Default network port
            sea_level_pressure: 1013.25, // Default sea level pressure
            i2c_address_decimal: 0x77, // Default I2C address (119 in decimal)
            i2c_bus_device_path: String::from("/dev/i2c-1"), // Default I2C bus device path
            bind_address: String::from("0.0.0.0"), // Default bind address
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
    temperature: f32,
    humidity: f32,
    pressure: f32,
    altitude: f32,
}

async fn get_sensor_data() -> impl Responder {
    let config = read_or_create_config();

    // Set up the I2C bus and BME280 sensor
    let i2c_bus = match I2cdev::new(&config.i2c_bus_device_path) {
        Ok(bus) => bus,
        Err(_) => return HttpResponse::InternalServerError().body("Failed to open I2C bus"),
    };

    // Create the delay object from linux_embedded_hal
    let mut delay = Delay {};

    // Create BME280 sensor object with the correct I2C address
    let mut bme280 = BME280::new(i2c_bus, config.i2c_address_decimal as u8);

    // Initialize the BME280 sensor with the delay
    if let Err(e) = bme280.init(&mut delay) {
        eprintln!("Failed to initialize BME280 sensor: {:?}", e);
        return HttpResponse::InternalServerError().body("Failed to initialize BME280 sensor");
    }

    // Read sensor data with the delay
    let data = match bme280.measure(&mut delay) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Failed to read sensor data: {:?}", e);
            return HttpResponse::InternalServerError().body("Failed to read sensor data");
        }
    };

    // Convert raw data to values
    let temperature = data.temperature;
    let humidity = data.humidity;
    let pressure = data.pressure / 100.0; // Convert pressure from Pa to hPa

    // Calculate altitude using the formula:
    // altitude = 44330 * (1.0 - (pressure / config.sea_level_pressure).powf(1.0 / 5.255))
    let altitude = 44330.0 * (1.0 - (pressure / config.sea_level_pressure).powf(1.0 / 5.255));

    // Create sensor data response
    let sensor_data = SensorData {
        timestamp: chrono::Utc::now().to_rfc3339(),
        model: String::from("BME280"),
        temperature,
        humidity,
        pressure,
        altitude,
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