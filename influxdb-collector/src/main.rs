mod api;
mod config;
mod influxdb;
mod utils;

use crate::config::create_config;
use crate::api::fetch_sensor_data;
use crate::influxdb::send_log;
//use crate::utils::{log_error, log_info};
use std::sync::{Arc, Mutex};
use tokio::time::{interval, Duration};
use tokio_retry::strategy::{ExponentialBackoff, jitter};
use tokio_retry::Retry;

#[tokio::main]
async fn main() {
    // Load or create the configuration
    let config = create_config();
    let config = Arc::new(config);

    // Create an interval based on the config setting
    let mut interval = interval(Duration::from_secs(config.query_interval));

    loop {
        interval.tick().await;

        for api_url in &config.api_urls {
            let api_url = api_url.clone();
            let config = Arc::clone(&config);

            // Retry fetching sensor data from the API
            let sensor_data = match Retry::spawn(ExponentialBackoff::from_millis(10).map(jitter).take(5), {
                let api_url = api_url.clone();
                move || {
                    let api_url = api_url.clone();
                    tokio::task::spawn_blocking(move || fetch_sensor_data(&api_url))
                }
            })
            .await {
                Ok(data) => data,
                Err(_) => {
                    //log_error(&format!("Failed to fetch sensor data from {} after retries", api_url));
                    continue; // Skip to the next API URL
                }
            };

            let influxdb_url = config.influxdb_url.clone();
            let influxdb_api_key = config.influxdb_api_key.clone();
            let influxdb_org = config.influxdb_org.clone();
            let influxdb_bucket = config.influxdb_bucket.clone();
            let sensor_data = Arc::new(Mutex::new(sensor_data));

            // Retry sending the logs to InfluxDB
            if let Err(_) = Retry::spawn(ExponentialBackoff::from_millis(10).map(jitter).take(5), {
                let influxdb_url = influxdb_url.clone();
                let influxdb_api_key = influxdb_api_key.clone();
                let influxdb_org = influxdb_org.clone();
                let influxdb_bucket = influxdb_bucket.clone();
                let sensor_data = Arc::clone(&sensor_data);
                move || {
                    let influxdb_url = influxdb_url.clone();
                    let influxdb_api_key = influxdb_api_key.clone();
                    let influxdb_org = influxdb_org.clone();
                    let influxdb_bucket = influxdb_bucket.clone();
                    let sensor_data = Arc::clone(&sensor_data);
                    tokio::task::spawn_blocking(move || {
                        let sensor_data = sensor_data.lock().unwrap();
                        send_log(&influxdb_url, &influxdb_api_key, &influxdb_org, &influxdb_bucket, &sensor_data)
                    })
                }
            })
            .await {
                //log_error("Failed to send log to InfluxDB after retries");
                continue; // Skip to the next API URL
            }

            //log_info(&format!("Successfully fetched sensor data from {} and sent log to InfluxDB", api_url));
        }
    }
}