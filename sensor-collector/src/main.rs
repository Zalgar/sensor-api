mod api;
mod config;
mod influxdb;
mod mqtt;
mod utils;

use crate::config::create_config;
use crate::api::fetch_sensor_data;
use crate::influxdb::send_log;
use crate::mqtt::send_mqtt_sensor_data;
use crate::utils::Logger;
use std::sync::{Arc, Mutex};
use tokio::time::{interval, Duration};
use tokio_retry::strategy::{ExponentialBackoff, jitter};
use tokio_retry::Retry;

#[tokio::main]
async fn main() {
    // Load or create the configuration
    let config = create_config();
    let logger = Logger::new(&config);
    let config = Arc::new(config);
    let logger = Arc::new(logger);

    logger.info("main", "Starting sensor data collector");
    logger.info("main", &format!("Query interval: {} seconds", config.query_interval));
    logger.info("main", &format!("InfluxDB enabled: {}", config.enable_influxdb));
    logger.info("main", &format!("MQTT enabled: {}", config.enable_mqtt));

    // Create an interval based on the config setting
    let mut interval = interval(Duration::from_secs(config.query_interval));

    loop {
        interval.tick().await;

        for api_url in &config.api_urls {
            let api_url = api_url.clone();
            let config = Arc::clone(&config);
            let logger = Arc::clone(&logger);

            logger.debug("main", &format!("Processing API URL: {}", api_url));

            // Retry fetching sensor data from the API
            let sensor_data = match Retry::spawn(ExponentialBackoff::from_millis(10).map(jitter).take(5), {
                let api_url = api_url.clone();
                let logger = Arc::clone(&logger);
                move || {
                    let api_url = api_url.clone();
                    let logger = Arc::clone(&logger);
                    tokio::task::spawn_blocking(move || fetch_sensor_data(&api_url, &logger))
                }
            })
            .await {
                Ok(Ok(data)) => data,
                Ok(Err(e)) => {
                    logger.error("main", &format!("Failed to fetch sensor data from {}: {}", api_url, e));
                    continue; // Skip to the next API URL
                }
                Err(_) => {
                    logger.error("main", &format!("Failed to fetch sensor data from {} after retries", api_url));
                    continue; // Skip to the next API URL
                }
            };

            let influxdb_url = config.influxdb_url.clone();
            let influxdb_api_key = config.influxdb_api_key.clone();
            let influxdb_org = config.influxdb_org.clone();
            let influxdb_bucket = config.influxdb_bucket.clone();
            let sensor_data = Arc::new(Mutex::new(sensor_data));

            // Send to InfluxDB if enabled
            if config.enable_influxdb {
                logger.debug("main", "InfluxDB is enabled, preparing to send data");
                
                // Retry sending the logs to InfluxDB
                if let Err(_) = Retry::spawn(ExponentialBackoff::from_millis(10).map(jitter).take(5), {
                    let influxdb_url = influxdb_url.clone();
                    let influxdb_api_key = influxdb_api_key.clone();
                    let influxdb_org = influxdb_org.clone();
                    let influxdb_bucket = influxdb_bucket.clone();
                    let sensor_data = Arc::clone(&sensor_data);
                    let logger = Arc::clone(&logger);
                    move || {
                        let influxdb_url = influxdb_url.clone();
                        let influxdb_api_key = influxdb_api_key.clone();
                        let influxdb_org = influxdb_org.clone();
                        let influxdb_bucket = influxdb_bucket.clone();
                        let sensor_data = Arc::clone(&sensor_data);
                        let logger = Arc::clone(&logger);
                        tokio::task::spawn_blocking(move || {
                            let sensor_data = sensor_data.lock().unwrap();
                            send_log(&influxdb_url, &influxdb_api_key, &influxdb_org, &influxdb_bucket, &sensor_data, &logger)
                        })
                    }
                })
                .await {
                    logger.error("main", "Failed to send log to InfluxDB after retries");
                }
            } else {
                logger.debug("main", "InfluxDB is disabled, skipping InfluxDB send");
            }

            // Send to MQTT if enabled
            if config.enable_mqtt {
                logger.debug("main", "MQTT is enabled, preparing to send data");
                let mqtt_broker = config.mqtt_broker.clone();
                let mqtt_port = config.mqtt_port;
                let mqtt_client_id = config.mqtt_client_id.clone();
                let mqtt_device_id = config.mqtt_device_id.clone();
                let mqtt_username = config.mqtt_username.clone();
                let mqtt_password = config.mqtt_password.clone();
                
                logger.info("main", &format!("Attempting MQTT publish to {}:{}", mqtt_broker, mqtt_port));
                
                // Retry sending to MQTT
                if let Err(e) = Retry::spawn(ExponentialBackoff::from_millis(10).map(jitter).take(5), {
                    let mqtt_broker = mqtt_broker.clone();
                    let mqtt_client_id = mqtt_client_id.clone();
                    let mqtt_device_id = mqtt_device_id.clone();
                    let mqtt_username = mqtt_username.clone();
                    let mqtt_password = mqtt_password.clone();
                    let sensor_data = Arc::clone(&sensor_data);
                    let logger = Arc::clone(&logger);
                    move || {
                        let mqtt_broker = mqtt_broker.clone();
                        let mqtt_client_id = mqtt_client_id.clone();
                        let mqtt_device_id = mqtt_device_id.clone();
                        let mqtt_username = mqtt_username.clone();
                        let mqtt_password = mqtt_password.clone();
                        let sensor_data = Arc::clone(&sensor_data);
                        let logger = Arc::clone(&logger);
                        tokio::task::spawn_blocking(move || {
                            let sensor_data = sensor_data.lock().unwrap();
                            send_mqtt_sensor_data(
                                &mqtt_broker,
                                mqtt_port,
                                &mqtt_client_id,
                                &mqtt_device_id,
                                mqtt_username.as_deref(),
                                mqtt_password.as_deref(),
                                &sensor_data,
                                &logger,
                            )
                        })
                    }
                })
                .await {
                    logger.error("main", &format!("Failed to send message to MQTT after retries: {:?}", e));
                } else {
                    logger.info("main", "MQTT message sent successfully");
                }
            } else {
                logger.debug("main", "MQTT is disabled, skipping MQTT publish");
            }

            //log_info(&format!("Successfully processed sensor data from {}", api_url));
        }
    }
}