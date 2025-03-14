use reqwest::blocking::Client;
use serde_json::Value;

pub fn fetch_sensor_data(api_url: &str) -> Value {
    let client = Client::new();
    let response = client.get(api_url).send().expect("Failed to fetch sensor data");
    response.json().expect("Failed to parse sensor data")
}