use actix_web::{web, App, HttpServer, HttpResponse, Responder, middleware::Logger}; // Import necessary Actix Web components
use tsl2591::{Driver, Gain, IntegrationTimes}; // Import TSL2591 sensor library and necessary enums
use linux_embedded_hal::{I2cdev, Delay};  // Import I2C device and delay from linux_embedded_hal
use serde::{Deserialize, Serialize}; // Import serialization/deserialization from Serde
use chrono::Utc; // Import Utc for timestamps
use std::fs; // Import filesystem operations
use std::fs::File; // Import file operations
use std::io::Write; // Import write operations
use env_logger::Env; // Import environment logger

// Configuration structure for the application
#[derive(Serialize, Deserialize)]
struct Config {
    network_port: u16, // Port for the web server
    i2c_address_decimal: u16, // I2C address of the TSL2591 sensor
    i2c_bus_device_path: String, // Path to the I2C bus device
    bind_address: String, // Address to bind the web server to
}

// Default implementation for the Config struct
impl Default for Config {
    fn default() -> Self {
        Config {
            network_port: 5004, // Default network port
            i2c_address_decimal: 0x29, // Default I2C address (41 in decimal)
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
    luminosity: f32,
}

async fn get_sensor_data() -> impl Responder {
    let config = read_or_create_config();

    // Set up the I2C bus and TSL2591 sensor
    let i2c_bus = match I2cdev::new(&config.i2c_bus_device_path) {
        Ok(bus) => bus,
        Err(_) => return HttpResponse::InternalServerError().body("Failed to open I2C bus"),
    };

    // Create the delay object from linux_embedded_hal
    let mut delay = Delay {};

    // Create TSL2591 sensor object with the correct I2C address
    let mut tsl2591 = Driver::new(i2c_bus).unwrap();

    // Initialize the TSL2591 sensor with the delay
    tsl2591.enable().unwrap();

    // Set gain and integration time
    tsl2591.set_gain(Some(Gain::MED)).unwrap();
    tsl2591.set_timing(Some(IntegrationTimes::_100MS)).unwrap();

    // Read sensor data
    let (ch0, ch1) = match tsl2591.get_channel_data(&mut delay) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Failed to read sensor data: {:?}", e);
            return HttpResponse::InternalServerError().body("Failed to read sensor data");
        }
    };

    let luminosity = match tsl2591.calculate_lux(ch0, ch1) {
        Ok(lux) => lux,
        Err(e) => {
            eprintln!("Failed to calculate lux: {:?}", e);
            return HttpResponse::InternalServerError().body("Failed to calculate lux");
        }
    };

    // Create sensor data response
    let sensor_data = SensorData {
        timestamp: Utc::now().to_rfc3339(),
        model: String::from("TSL2591"),
        luminosity,
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