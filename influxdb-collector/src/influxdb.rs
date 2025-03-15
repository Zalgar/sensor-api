use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use serde_json::Value;
use crate::utils::get_hostname;

pub fn send_log(influxdb_url: &str, api_key: &str, org: &str, bucket: &str, sensor_data: &Value) {
    let client = Client::new();
    let mut headers = HeaderMap::new();
    headers.insert(AUTHORIZATION, HeaderValue::from_str(&format!("Token {}", api_key)).unwrap());

    let write_url = format!("{}/api/v2/write?org={}&bucket={}&precision=s", influxdb_url, org, bucket);

    // Convert sensor data to line protocol format
    let line_protocol = convert_to_line_protocol(sensor_data);

    let response = client.post(&write_url)
        .headers(headers)
        .body(line_protocol)
        .send()
        .expect("Failed to send log to InfluxDB");

    if !response.status().is_success() {
        eprintln!("Failed to send log to InfluxDB: {}", response.status());
    }
}

fn convert_to_line_protocol(sensor_data: &Value) -> String {
    let hostname = get_hostname();
    let measurement = sensor_data.get("model").and_then(Value::as_str).unwrap_or("unknown");

    // Assuming sensor_data is a JSON object with key-value pairs
    let mut lines = Vec::new();
    if let Some(obj) = sensor_data.as_object() {
        for (key, value) in obj {
            let line = format!("{},host={} {}={}", measurement, hostname, key, value);
            lines.push(line);
        }
    }
    lines.join("\n")
}