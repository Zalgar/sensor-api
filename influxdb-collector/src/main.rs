mod api;
mod config;
mod influxdb;
mod utils;

use crate::config::create_config;
use crate::api::fetch_sensor_data;
use crate::influxdb::send_log;
use crate::utils::{log_error, log_info};
use tokio::time::{interval, Duration};
use tokio_retry::strategy::{ExponentialBackoff, jitter};
use tokio_retry::Retry;

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

        // Retry fetching sensor data from the API
        let sensor_data = Retry::spawn(ExponentialBackoff::from_millis(10).map(jitter).take(5), move || {
            let api_url = api_url.clone();
            tokio::task::spawn_blocking(move || fetch_sensor_data(&api_url))
        })
        .await
        .unwrap_or_else(|_| {
            log_error("Failed to fetch sensor data after retries");
            panic!("Failed to fetch sensor data after retries");
        });

        // Retry sending the logs to InfluxDB
        Retry::spawn(ExponentialBackoff::from_millis(10).map(jitter).take(5), move || {
            let influxdb_url = influxdb_url.clone();
            let influxdb_api_key = influxdb_api_key.clone();
            let influxdb_org = influxdb_org.clone();
            let influxdb_bucket = influxdb_bucket.clone();
            let sensor_data = sensor_data.clone();
            tokio::task::spawn_blocking(move || {
                send_log(&influxdb_url, &influxdb_api_key, &influxdb_org, &influxdb_bucket, &sensor_data)
            })
        })
        .await
        .unwrap_or_else(|_| {
            log_error("Failed to send log to InfluxDB after retries");
            panic!("Failed to send log to InfluxDB after retries");
        });

        log_info("Successfully fetched sensor data and sent log to InfluxDB");
    }
}