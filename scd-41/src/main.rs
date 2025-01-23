use actix_web::{web, App, HttpServer, HttpResponse, middleware::Logger}; // Import necessary Actix Web components
use scd4x::Scd4x; // Import SCD-41 sensor library
use linux_embedded_hal::{I2cdev, Delay, I2CError};  // Import I2C device, delay, and I2CError from linux_embedded_hal
use linux_embedded_hal::i2cdev::linux::LinuxI2CError; // Import LinuxI2CError from linux_embedded_hal
use serde::{Deserialize, Serialize}; // Import serialization/deserialization from Serde
use chrono::Utc; // Import Utc for timestamps
use std::fs; // Import filesystem operations
use std::fs::File; // Import file operations
use std::io::Write; // Import write operations
use env_logger::Env; // Import environment logger
use std::fmt; // Import fmt for custom error formatting

// Configuration structure for the application
#[derive(Serialize, Deserialize)]
struct Config {
    network_port: u16, // Port for the web server
    i2c_address_decimal: u16, // I2C address of the SCD-41 sensor
    i2c_bus_device_path: String, // Path to the I2C bus device
    bind_address: String, // Address to bind the web server to
}

// Default implementation for the Config struct
impl Default for Config {
    fn default() -> Self {
        Config {
            network_port: 5000, // Default network port
            i2c_address_decimal: 0x62, // Default I2C address (98 in decimal)
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
    co2: f32,
}

// Define a custom error type
#[derive(Debug)]
enum SensorError {
    I2cError(I2CError),
    Scd4xError(scd4x::Error<I2CError>),
    LinuxI2CError(LinuxI2CError),
}

impl fmt::Display for SensorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SensorError::I2cError(e) => write!(f, "I2C error: {}", e),
            SensorError::Scd4xError(e) => write!(f, "SCD4x error: {:?}", e),
            SensorError::LinuxI2CError(e) => write!(f, "Linux I2C error: {:?}", e),
        }
    }
}

impl std::error::Error for SensorError {}

impl From<I2CError> for SensorError {
    fn from(err: I2CError) -> SensorError {
        SensorError::I2cError(err)
    }
}

impl From<scd4x::Error<I2CError>> for SensorError {
    fn from(err: scd4x::Error<I2CError>) -> SensorError {
        SensorError::Scd4xError(err)
    }
}

impl From<LinuxI2CError> for SensorError {
    fn from(err: LinuxI2CError) -> SensorError {
        SensorError::LinuxI2CError(err)
    }
}

// Function to read data from the SCD-41 sensor
fn read_sensor_data(config: &Config) -> Result<SensorData, SensorError> {
    let i2c_bus = I2cdev::new(&config.i2c_bus_device_path)?;
    let mut sensor = Scd4x::new(i2c_bus, Delay);
    
    // Stop any ongoing measurement
    sensor.stop_periodic_measurement()?;
    std::thread::sleep(std::time::Duration::from_secs(1)); // Wait for the sensor to stop

    // Start a new periodic measurement
    sensor.start_periodic_measurement()?;
    std::thread::sleep(std::time::Duration::from_secs(5)); // Wait for the first measurement

    let data = sensor.measurement()?;
    Ok(SensorData {
        timestamp: Utc::now().to_rfc3339(),
        model: String::from("SCD-41"),
        temperature: data.temperature,
        humidity: data.humidity,
        co2: data.co2 as f32, // Convert u16 to f32
    })
}

// Main function to start the web server
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(Env::default().default_filter_or("info"));
    let config = read_or_create_config();

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .route("/sensor_data", web::get().to(|| async {
                let config = read_or_create_config();
                match read_sensor_data(&config) {
                    Ok(sensor_data) => HttpResponse::Ok().json(sensor_data),
                    Err(e) => HttpResponse::InternalServerError().body(format!("Error reading sensor data: {}", e)),
                }
            }))
    })
    .bind((config.bind_address.as_str(), config.network_port))?
    .run()
    .await
}