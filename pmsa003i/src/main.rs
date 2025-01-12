use actix_web::{web, App, HttpServer, Responder, HttpResponse};
use pmsa003i::Pmsa003i;
use linux_embedded_hal::I2cdev;
use serde::Serialize;
use chrono::Utc;

#[derive(Serialize)]
struct SensorData {
    pm1_0: u16,
    pm2_5: u16,
    pm10: u16,
    model: String,
    timestamp: String,
}

async fn get_sensor_data() -> impl Responder {
    // Set up the I2C bus and PMSA003I sensor
    let i2c_bus = match I2cdev::new("/dev/i2c-1") {
        Ok(bus) => bus,
        Err(_) => return HttpResponse::InternalServerError().body("Failed to open I2C bus"),
    };

    // Create the delay object from linux_embedded_hal
    let _delay = linux_embedded_hal::Delay {};

    // Create PMSA003I sensor object
    let mut pmsa003i = Pmsa003i::new(i2c_bus);

    // Read sensor data
    match pmsa003i.read() {
        Ok(data) => {
            // Extract particulate matter concentrations
            let pm1_0 = data.pm1;
            let pm2_5 = data.pm2_5;
            let pm10 = data.pm10;

            // Create the response struct
            let response = SensorData {
                timestamp: Utc::now().to_rfc3339(),
                model: "PMSA003I".to_string(),
                pm1_0,
                pm2_5,
                pm10,
            };

            // Return the data as JSON
            HttpResponse::Ok().json(response)
        }
        Err(e) => {
            // Log the error and return an error response
            eprintln!("Failed to read sensor data: {:?}", e);
            HttpResponse::InternalServerError().body("Failed to read sensor data")
        }
    }
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