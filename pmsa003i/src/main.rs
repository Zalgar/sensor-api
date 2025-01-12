use actix_web::{web, App, HttpServer, Responder};
use pmsa003i::Pmsa003i;
use linux_embedded_hal::I2cdev;
use serde::Serialize;

#[derive(Serialize)]
struct SensorData {
    pm1_0: u16,
    pm2_5: u16,
    pm10: u16,
}

async fn get_sensor_data() -> impl Responder {
    // Set up the I2C bus and PMSA003I sensor
    let i2c_bus = I2cdev::new("/dev/i2c-1").expect("Failed to open I2C bus");

    // Create the delay object from linux_embedded_hal
    let _delay = linux_embedded_hal::Delay {};

    // Create PMSA003I sensor object
    let mut pmsa003i = Pmsa003i::new(i2c_bus);

    // Read sensor data
    let data = pmsa003i.read().expect("Failed to read sensor data");

    // Extract particulate matter concentrations
    let pm1_0 = data.pm1;
    let pm2_5 = data.pm2_5;
    let pm10 = data.pm10;

    // Create the response struct
    let response = SensorData {
        pm1_0,
        pm2_5,
        pm10,
    };

    // Return the data as JSON
    web::Json(response)
}

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