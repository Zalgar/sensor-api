use reqwest::blocking::Client;
use serde_json::Value;
use crate::utils::Logger;

pub fn fetch_sensor_data(api_url: &str, logger: &Logger) -> Result<Value, String> {
    logger.debug("api", &format!("Fetching sensor data from: {}", api_url));
    
    let client = Client::new();
    
    // Send the request
    let response = client.get(api_url)
        .send()
        .map_err(|e| {
            let error_msg = format!("Failed to fetch from {}: {}", api_url, e);
            logger.error("api", &error_msg);
            error_msg
        })?;
    
    // Check if the response was successful
    if !response.status().is_success() {
        let error_msg = format!("HTTP error {} from {}", response.status(), api_url);
        logger.error("api", &error_msg);
        return Err(error_msg);
    }
    
    // Get the response text first to debug what we're receiving
    let response_text = response.text()
        .map_err(|e| {
            let error_msg = format!("Failed to read response body from {}: {}", api_url, e);
            logger.error("api", &error_msg);
            error_msg
        })?;
    
    // Check if the response is empty
    if response_text.trim().is_empty() {
        let error_msg = format!("Empty response from {}", api_url);
        logger.error("api", &error_msg);
        return Err(error_msg);
    }
    
    logger.debug("api", &format!("Response from {}: {}", api_url, 
        response_text.chars().take(100).collect::<String>()));
    
    // Try to parse as JSON and provide better error context
    let result = serde_json::from_str(&response_text)
        .map_err(|e| {
            let error_msg = format!("Failed to parse JSON from {}: {}. Response was: '{}'", 
                                api_url, e, response_text.chars().take(200).collect::<String>());
            logger.error("api", &error_msg);
            error_msg
        })?;
    
    logger.info("api", &format!("Successfully fetched sensor data from {}", api_url));
    Ok(result)
}