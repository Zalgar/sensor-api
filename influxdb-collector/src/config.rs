use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub api_url: String,
    pub influxdb_url: String,
    pub influxdb_api_key: String,
    pub influxdb_org: String,
    pub influxdb_bucket: String,
    pub query_interval: u64, // Interval in seconds
}

pub fn create_config() -> Config {
    let config_path = "config.json";
    if !Path::new(config_path).exists() {
        let default_config = Config {
            api_url: "http://localhost:5000/sensor_data".to_string(),
            influxdb_url: "http://localhost:8086".to_string(),
            influxdb_api_key: "your_api_key".to_string(),
            influxdb_org: "your_org".to_string(),
            influxdb_bucket: "your_bucket".to_string(),
            query_interval: 60, // Default interval of 60 seconds
        };
        let config_data = serde_json::to_string_pretty(&default_config).unwrap();
        fs::write(config_path, config_data).expect("Unable to write config file");
    }

    let config_data = fs::read_to_string(config_path).expect("Unable to read config file");
    serde_json::from_str(&config_data).expect("Unable to parse config file")
}