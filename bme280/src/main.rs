use actix_web::{web, App, HttpServer, Responder, HttpResponse};
use bme280::i2c::BME280;
use linux_embedded_hal::{I2cdev, Delay};  // Import Delay from linux_embedded_hal
use serde::Serialize;
use chrono::Utc;

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
    // Set up the I2C bus and BME280 sensor
    let i2c_bus = match I2cdev::new("/dev/i2c-1") {
        Ok(bus) => bus,
        Err(_) => return HttpResponse::InternalServerError().body("Failed to open I2C bus"),
    };

    // Create the delay object from linux_embedded_hal
    let mut delay = Delay {};

    // Create BME280 sensor object with the correct I2C address (0x76 or 0x77)
    let mut bme280 = BME280::new(i2c_bus, 0x77);

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
    // altitude = 44330 * (1.0 - (pressure / sea_level_pressure).powf(1.0 / 5.255))
    let sea_level_pressure = 1013.25; // Default sea level pressure in hPa
    let altitude = 44330.0 * (1.0 - (pressure / sea_level_pressure).powf(1.0 / 5.255));

    // Create the response struct
    let response = SensorData {
        timestamp: Utc::now().to_rfc3339(),
        model: "BME280".to_string(),
        temperature,
        humidity,
        pressure,
        altitude,
    };

    // Return the response as JSON
    HttpResponse::Ok().json(response)}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Start the web server
    HttpServer::new(|| {
        App::new().route("/sensor_data", web::get().to(get_sensor_data))
    })
    .bind("0.0.0.0:5000")?
    .run()
    .await
}