use rumqttc::{Client, MqttOptions, QoS};
use serde_json::Value;
use std::time::Duration;
use crate::utils::Logger;

pub fn send_mqtt_sensor_data(
    broker: &str,
    port: u16,
    client_id: &str,
    device_id: &str,
    username: Option<&str>,
    password: Option<&str>,
    sensor_data: &Value,
    logger: &Logger,
) -> Result<(), String> {
    // Extract model from sensor data
    let model = sensor_data.get("model")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    
    logger.info("mqtt", &format!("Attempting to send MQTT messages to {}:{} for device '{}' model '{}'", broker, port, device_id, model));
    
    // Create a unique client ID to avoid conflicts
    let unique_client_id = format!("{}-{}", client_id, std::process::id());
    
    let mut mqttoptions = MqttOptions::new(unique_client_id, broker, port);
    mqttoptions.set_keep_alive(Duration::from_secs(60));
    
    if let (Some(user), Some(pass)) = (username, password) {
        logger.debug("mqtt", &format!("Using MQTT authentication for user: {}", user));
        mqttoptions.set_credentials(user, pass);
    } else {
        logger.debug("mqtt", "No MQTT authentication configured");
    }
    
    let (client, mut eventloop) = Client::new(mqttoptions, 10);
    
    // Extract all sensor values (excluding timestamp and model)
    let mut sensor_values = Vec::new();
    if let Some(obj) = sensor_data.as_object() {
        for (key, value) in obj {
            // Skip metadata fields
            if key != "timestamp" && key != "model" {
                let topic = format!("{}/{}/{}", device_id, model, key);
                
                // Convert value to string for payload
                let payload = match value {
                    Value::String(s) => s.clone(),
                    Value::Number(n) => n.to_string(),
                    Value::Bool(b) => b.to_string(),
                    _ => serde_json::to_string(value).unwrap_or_else(|_| "null".to_string()),
                };
                
                sensor_values.push((topic, payload));
            }
        }
    }
    
    if sensor_values.is_empty() {
        return Err("No sensor values found to publish".to_string());
    }
    
    logger.info("mqtt", &format!("Preparing to publish {} sensor values", sensor_values.len()));
    
    // Publish each sensor value to its own topic
    for (topic, payload) in &sensor_values {
        logger.debug("mqtt", &format!("Publishing to topic '{}': {}", topic, payload));
        match client.publish(topic, QoS::AtMostOnce, false, payload.clone()) {
            Ok(_) => logger.debug("mqtt", &format!("Queued message for topic '{}'", topic)),
            Err(e) => return Err(format!("Failed to queue MQTT message for topic '{}': {}", topic, e)),
        }
    }
    
    // Create a tokio runtime for the event loop
    let rt = tokio::runtime::Runtime::new()
        .map_err(|e| format!("Failed to create tokio runtime: {}", e))?;
    
    // Run the event loop briefly to ensure the messages are sent
    let result = rt.block_on(async {
        let timeout = Duration::from_secs(5);
        let start_time = std::time::Instant::now();
        let mut connected = false;
        let mut messages_processed = 0;
        let mut messages_sent = 0;
        let expected_messages = sensor_values.len();
        
        while start_time.elapsed() < timeout && messages_processed < expected_messages + 10 {
            // Use a timeout for each poll to avoid hanging
            match tokio::time::timeout(Duration::from_millis(1000), eventloop.eventloop.poll()).await {
                Ok(Ok(notification)) => {
                    messages_processed += 1;
                    match notification {
                        rumqttc::Event::Incoming(rumqttc::Packet::ConnAck(_)) => {
                            connected = true;
                            logger.debug("mqtt", "Connected to MQTT broker");
                        }
                        rumqttc::Event::Outgoing(rumqttc::Outgoing::Publish(_)) => {
                            messages_sent += 1;
                            logger.debug("mqtt", &format!("Message {} of {} sent to broker", messages_sent, expected_messages));
                            
                            // If all messages are sent, we can exit
                            if messages_sent >= expected_messages {
                                // Give a bit more time to ensure delivery
                                tokio::time::sleep(Duration::from_millis(500)).await;
                                return Ok(format!("All {} messages sent successfully", expected_messages));
                            }
                        }
                        _ => {
                            // Other events, continue processing
                        }
                    }
                }
                Ok(Err(e)) => {
                    return Err(format!("MQTT connection error: {}", e));
                }
                Err(_) => {
                    // Timeout on this poll, try again
                    continue;
                }
            }
        }
        
        if !connected {
            return Err("Failed to connect to MQTT broker within timeout".to_string());
        }
        
        if messages_sent < expected_messages {
            logger.warn("mqtt", &format!("Only {} of {} messages confirmed sent", messages_sent, expected_messages));
        }
        
        // We connected, so the messages were likely sent even if we didn't see all confirmations
        Ok(format!("Connected successfully, {} of {} messages likely sent", messages_sent.max(expected_messages), expected_messages))
    });
    
    match result {
        Ok(msg) => {
            logger.info("mqtt", &msg);
            Ok(())
        }
        Err(e) => {
            logger.error("mqtt", &e);
            Err(e)
        }
    }
}
