use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub api_urls: Vec<String>, // Changed from a single URL to a list of URLs
    pub influxdb_url: String,
    pub influxdb_api_key: String,
    pub influxdb_org: String,
    pub influxdb_bucket: String,
    pub query_interval: u64, // Interval in seconds
    
    // Output configuration flags
    pub enable_influxdb: bool,
    pub enable_mqtt: bool,
    
    // Logging configuration
    pub log_level: String, // "debug", "info", "warn", "error"
    pub enable_mqtt_debug: bool,
    pub enable_influxdb_debug: bool,
    pub enable_api_debug: bool,
    
    // MQTT configuration
    pub mqtt_broker: String,
    pub mqtt_port: u16,
    pub mqtt_device_id: String, // Device identifier (e.g., "sensor-collector")
    pub mqtt_client_id: String,
    pub mqtt_username: Option<String>,
    pub mqtt_password: Option<String>,
}

pub fn create_config() -> Config {
    let config_path = "config.json";
    if !Path::new(config_path).exists() {
        let default_config = Config {
            api_urls: vec!["http://localhost:5000/sensor_data".to_string()], // Default list of URLs
            influxdb_url: "http://localhost:8086".to_string(),
            influxdb_api_key: "your_api_key".to_string(),
            influxdb_org: "your_org".to_string(),
            influxdb_bucket: "your_bucket".to_string(),
            query_interval: 60, // Default interval of 60 seconds
            
            // Output configuration flags
            enable_influxdb: true,
            enable_mqtt: false,
            
            // Logging configuration
            log_level: "info".to_string(), // Default to info level
            enable_mqtt_debug: false,
            enable_influxdb_debug: false,
            enable_api_debug: false,
            
            // MQTT configuration
            mqtt_broker: "localhost".to_string(),
            mqtt_port: 1883,
            mqtt_device_id: "sensor-collector".to_string(), // Device identifier for topic structure
            mqtt_client_id: "sensor-collector".to_string(),
            mqtt_username: None,
            mqtt_password: None,
        };
        let config_data = serde_json::to_string_pretty(&default_config).unwrap();
        fs::write(config_path, config_data).expect("Unable to write config file");
    }

    let config_data = fs::read_to_string(config_path).expect("Unable to read config file");
    serde_json::from_str(&config_data).expect("Unable to parse config file")
}