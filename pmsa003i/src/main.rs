use actix_web::{web, App, HttpServer, HttpResponse, Responder, middleware::Logger};
use env_logger::Env;
use serde::{Deserialize, Serialize};
use chrono::Utc;
//use std::fs;
use std::fs::File;
use std::io::Write;
use linux_embedded_hal::I2cdev;
use pmsa003i::Pmsa003i;

#[derive(Serialize, Deserialize)]
struct Config {
    network_port: u16,
    i2c_bus_device_path: String,
    bind_address: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            network_port: 5001,
            i2c_bus_device_path: String::from("/dev/i2c-1"),
            bind_address: String::from("0.0.0.0"),
        }
    }
}

fn read_or_create_config() -> Config {
    let config_path = "config.json";
    if let Ok(file) = File::open(config_path) {
        serde_json::from_reader(file).unwrap_or_else(|_| Config::default())
    } else {
        let default_config = Config::default();
        let config_data = serde_json::to_string_pretty(&default_config).unwrap();
        let mut file = File::create(config_path).unwrap();
        file.write_all(config_data.as_bytes()).unwrap();
        default_config
    }
}

#[derive(Serialize)]
struct SensorData {
    timestamp: String,
    model: String,
    pm1_0: u16,
    pm2_5: u16,
    pm10: u16,   
}

async fn get_sensor_data() -> impl Responder {
    let config = read_or_create_config();

    let i2c_bus = match I2cdev::new(&config.i2c_bus_device_path) {
        Ok(bus) => bus,
        Err(_) => return HttpResponse::InternalServerError().body("Failed to open I2C bus"),
    };

    let mut pmsa003i = Pmsa003i::new(i2c_bus);

    match pmsa003i.read() {
        Ok(data) => {
            let pm1_0 = data.pm1;
            let pm2_5 = data.pm2_5;
            let pm10 = data.pm10;

            let response = SensorData {
                timestamp: Utc::now().to_rfc3339(),
                model: "PMSA003I".to_string(),
                pm1_0,
                pm2_5,
                pm10,
            };

            HttpResponse::Ok().json(response)
        }
        Err(e) => {
            eprintln!("Failed to read sensor data: {:?}", e);
            HttpResponse::InternalServerError().body("Failed to read sensor data")
        }
    }
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
    .bind((config.bind_address.as_str(), config.network_port))?
    .run()
    .await
}