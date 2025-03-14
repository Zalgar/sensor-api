mod api;
mod config;
mod influxdb;
mod utils;

use crate::config::create_config;
use crate::api::fetch_sensor_data;
use crate::influxdb::send_log;
use tokio::time::{interval, Duration};

#[tokio::main]
async fn main() {
    // Load or create the configuration
    let config = create_config();

    // Create an interval based on the config setting
    let mut interval = interval(Duration::from_secs(config.query_interval));

    loop {
        interval.tick().await;

        // Clone the necessary config values
        let api_url = config.api_url.clone();
        let influxdb_url = config.influxdb_url.clone();
        let influxdb_api_key = config.influxdb_api_key.clone();
        let influxdb_org = config.influxdb_org.clone();
        let influxdb_bucket = config.influxdb_bucket.clone();

        // Fetch sensor data from the API in a blocking task
        let sensor_data = tokio::task::spawn_blocking(move || fetch_sensor_data(&api_url))
            .await
            .expect("Failed to fetch sensor data");

        // Send the logs to InfluxDB in a blocking task
        tokio::task::spawn_blocking(move || {
            send_log(&influxdb_url, &influxdb_api_key, &influxdb_org, &influxdb_bucket, &sensor_data)
        })
        .await
        .expect("Failed to send log to InfluxDB");
    }
}